use std::error::Error;
use std::sync::mpsc::{self, RecvError};
use std::thread;
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
}

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
                .default_value("-")
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
                .short("p")
                .long("test_prefix")
                .help("The prefix to search for to indicate something is a test")
                .default_value("test_")
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        watch: matches.is_present("watch"),
        test_prefix: matches.value_of("test_prefix").unwrap().to_string(),
    })
}

pub fn run(config: Config) -> Rysult<()> {
    let (tx_files, rx_files) = mpsc::channel();
    let (tx_tests, rx_tests) = mpsc::channel();
    let (tx_results, rx_results) = mpsc::channel();

    let _ = thread::spawn(move || {
        let tx_files = tx_files.clone();
        find_files(config.files.clone(), config.watch, tx_files).unwrap();
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

pub fn find_files(paths: Vec<String>, watch: bool, tx: mpsc::Sender<String>) -> Rysult<()> {
    for path in &paths {
        for entry in glob(path.as_str())? {
            match entry {
                Ok(p) => {
                    if p.is_file() && p.file_stem().unwrap().to_string_lossy().starts_with("test_") && p.extension().unwrap() == "py" {
                        println!("Would send file: {}", p.to_str().unwrap());
                        tx.send(p.to_str().unwrap().to_string())?;
                    }
                },
                Err(e) => println!("Error globbing: {}", e),
            }
        }
    }

    if watch {
        println!("Would watch files");
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
                println!("Would read file: {}", file_name);
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
                println!("Would run test: {}:{}", test.file, test.test);
                tx.send(test)?;
            },
            // TODO: Handle this better - should we be able to tell the difference between the channel being closed and an error?
            Err(RecvError) => break,
        }
    }

    Ok(())
}

pub fn output_results(rx: mpsc::Receiver<TestCase>) -> Rysult<()> {
    loop {
        match rx.recv() {
            Ok(result) => {
                println!("{}:{} - {}", result.file, result.test, result.passed);
            },
            // TODO: Handle this better - should we be able to tell the difference between the channel being closed and an error?
            Err(RecvError) => break,
        }
    }

    Ok(())
}