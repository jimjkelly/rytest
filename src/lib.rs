use std::error::Error;
use std::sync::mpsc::{self, RecvError};
use std::thread;
use std::time::Instant;
use clap::{App, Arg};
use std::fs::File;
use std::io::Read;
use rustpython_parser::{Parse, ast}; 
use rustpython_parser::ast::Stmt::FunctionDef;
use glob::glob;

type Rysult<T> = Result<T, Box<dyn Error>>;

pub struct Config {
    files: Vec<String>,
    watch: bool,
    test_prefix: String,
    file_prefix: String,
}

#[derive(Debug, PartialEq)]
pub struct TestCase {
    file: String,
    test: String,
    passed: bool,
}

pub fn get_args() -> Rysult<Config> {
    let matches = App::new("rytest")
        .version("0.1.0")
        .author("Jim Kelly <pthread1981@gmail.com>")
        .about("rytest is a reasonably fast Python test runner.")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value(".")
                .min_values(1)
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .help("Watch files for changes")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("test_prefix")
                .short("t")
                .long("test_prefix")
                .help("The prefix to search for to indicate a function is a test.  Defaults to test_")
                .default_value("test_")
        )
        .arg(
            Arg::with_name("file_prefix")
                .short("f")
                .long("file_prefix")
                .help("The prefix to search for to indicate a file contains tests.  Defaults to test_")
                .default_value("test_")
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        watch: matches.is_present("watch"),
        test_prefix: matches.value_of("test_prefix").unwrap().to_string(),
        file_prefix: matches.value_of("file_prefix").unwrap().to_string(),
    })
}

pub fn run(config: Config) -> Rysult<()> {
    let (tx_files, rx_files) = mpsc::channel();
    let (tx_tests, rx_tests) = mpsc::channel();
    let (tx_results, rx_results) = mpsc::channel();

    let paths = get_paths(config.files.clone());

    let _ = thread::spawn(move || {
        let tx_files = tx_files.clone();
        find_files(paths, config.file_prefix, config.watch, tx_files).unwrap();
    });

    let _ = thread::spawn(move || {
        let tx_tests = tx_tests.clone();
        find_tests(config.test_prefix.clone(), rx_files, tx_tests).unwrap();
    });

    let _ = thread::spawn(move || {
        let tx_results = tx_results.clone();
        run_tests(rx_tests, tx_results).unwrap();
    });

    let handle_output = thread::spawn(move || {
        let rx_results = rx_results;
        output_results(rx_results).unwrap();
    });

    handle_output.join().unwrap();

    Ok(())
}


pub fn get_paths(paths: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();

    for path in paths {
        if !path.ends_with("/**/*.py") {
            result.push(format!("{}/**/*.py", path));
        } else {
            result.push(path);
        }
    }

    result
}

pub fn find_files(paths: Vec<String>, file_prefix: String, watch: bool, tx: mpsc::Sender<String>) -> Rysult<()> {
    let file_prefix = file_prefix.as_str();

    for path in &paths {
        for entry in glob(path.as_str())? {
            match entry {
                Ok(p) => {
                    if p.is_file() && p.file_stem().unwrap().to_string_lossy().starts_with(file_prefix) && p.extension().unwrap() == "py" {
                        tx.send(p.to_str().unwrap().to_string())?;
                    }
                },
                Err(e) => println!("Error globbing: {}", e),
            }
        }
    }

    if watch {
        todo!("Implement watching files");
    }

    drop(tx);

    Ok(())
}

pub fn find_tests(prefix: String, rx: mpsc::Receiver<String>, tx: mpsc::Sender<TestCase>) -> Rysult<()> {
    loop {
        match rx.recv() {
            Ok(file_name) => {
                let mut data = String::new();
                let mut file = File::open(file_name.clone())?;
                file.read_to_string(&mut data)?;
                let ast = ast::Suite::parse(data.as_str(), "<embedded>");

                match ast {
                    Ok(ast) => {
                        for stmt in ast {
                            match stmt {
                                FunctionDef(node) if node.name.starts_with(&prefix) => tx.send(
                                    TestCase {
                                        file: file_name.clone(),
                                        test: node.name.to_string(),
                                        passed: false,
                                    }
                                )?,
                                _ => (),
                            }
                        }
                    
                    },
                    Err(e) => println!("Error parsing file: {}", e),
                }
            },
            // TODO: Handle this better - should we be able to tell the difference between the channel being closed and an error?
            Err(RecvError) => break,
        }
    }

    Ok(())
}

pub fn run_tests(rx: mpsc::Receiver<TestCase>, tx: mpsc::Sender<TestCase>) -> Rysult<()> {
    loop {
        match rx.recv() {
            Ok(mut test) => {
                // We should, ya know, actually run the test.
                test.passed = true;
                tx.send(test)?;
            },
            // TODO: Handle this better - should we be able to tell the difference between the channel being closed and an error?
            Err(RecvError) => break,
        }
    }

    Ok(())
}

pub fn output_results(rx: mpsc::Receiver<TestCase>) -> Rysult<()> {
    let start = Instant::now();
    let mut passed = 0;
    let mut failed = 0;

    loop {
        match rx.recv() {
            Ok(result) => {
                if result.passed {
                    passed += 1;
                } else {
                    failed += 1;
                }

                println!("{}:{} - {}", result.file, result.test, result.passed);
            },
            // TODO: Handle this better - should we be able to tell the difference between the channel being closed and an error?
            Err(RecvError) => break,
        }
    }

    let duration = start.elapsed();
    println!("{} passed, {} failed in {:?}", passed, failed, duration);

    Ok(())
}


#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use crate::TestCase;

    use super::{find_files, find_tests};

    pub struct Setup {
        expected_files: Vec<String>,
        expected_tests: Vec<TestCase>,
    }

    fn setup() -> Setup {
        Setup {
            expected_files: vec![
                "tests/input/folder/subfolder/test_subfolder_file.py".to_string(),
                "tests/input/folder/test_another_file.py".to_string(),
                "tests/input/test_bad_file.py".to_string(),
                "tests/input/test_file.py".to_string(),
            ],
            expected_tests: vec![
                TestCase {
                    file: "tests/input/folder/subfolder/test_subfolder_file.py".to_string(),
                    test: "test_something".to_string(),
                    passed: false,
                },
                TestCase {
                    file: "tests/input/folder/subfolder/test_subfolder_file.py".to_string(),
                    test: "test_false".to_string(),
                    passed: false,
                },
                TestCase {
                    file: "tests/input/folder/test_another_file.py".to_string(),
                    test: "test_another_function".to_string(),
                    passed: false,
                },
                TestCase {
                    file: "tests/input/folder/test_another_file.py".to_string(),
                    test: "test_function_with_decorator".to_string(),
                    passed: false,
                },
                TestCase {
                    file: "tests/input/test_file.py".to_string(),
                    test: "test_function_passes".to_string(),
                    passed: false,
                },
                TestCase {
                    file: "tests/input/test_file.py".to_string(),
                    test: "test_function_fails".to_string(),
                    passed: false,
                },
            ],
        }
    }

    #[test]
    fn test_get_paths() {
        let paths = vec!["tests/input".to_string(), "tests/input/**/*.py".to_string()];
        let result = super::get_paths(paths);

        assert_eq!(result, vec!["tests/input/**/*.py".to_string(), "tests/input/**/*.py".to_string()]);
    }

    #[test]
    fn test_find_files() {
        let (tx, rx) = mpsc::channel();
        let result = find_files(vec!["tests/input/**/*.py".to_string()], "test_".to_string(), false, tx);
        
        assert!(result.is_ok());

        let mut files = Vec::new();

        loop {
            match rx.recv() {
                Ok(file) => files.push(file),
                Err(_) => break,
            }
        }

        let setup = setup();

        assert_eq!(files, setup.expected_files);
    }

    #[test]
    fn test_find_tests() {
        let (tx_files, rx_files) = mpsc::channel();
        let (tx_tests, rx_tests) = mpsc::channel();

        let setup = setup();

        for file in setup.expected_files {
            tx_files.send(file).unwrap();
        }

        drop(tx_files);

        let result = find_tests("test_".to_string(), rx_files, tx_tests);
        
        assert!(result.is_ok());

        let mut tests = Vec::new();

        loop {
            match rx_tests.recv() {
                Ok(test) => tests.push(test),
                Err(_) => break,
            }
        }

        assert_eq!(tests, setup.expected_tests);
    }
}
