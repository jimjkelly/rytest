use std::error::Error;
use std::path::Path;
use std::sync::mpsc::{self, RecvError};
use std::{env, fs, thread};
use clap::{App, Arg};
use std::fs::File;
use std::io::Read;
use rustpython_parser::{Parse, ast}; 
use rustpython_parser::ast::Stmt::FunctionDef;
use glob::glob;

use pyo3::prelude::*;
use pyo3::types::PyList;

type Rysult<T> = Result<T, Box<dyn Error>>;

pub struct Config {
    files: Vec<String>,
    watch: bool,
    file_prefix: String,
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
        .about("rytest is a reasonably fast, somewhat Pytest compatible Python test runner.")
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
            Arg::with_name("file_prefix")
                .short("f")
                .long("file_prefix")
                .help("The prefix to search for to indicate a file contains tests")
                .default_value("test_")
        )
        .arg(
            Arg::with_name("test_prefix")
                .short("p")
                .long("test_prefix")
                .help("The prefix to search for to indicate a function is a test")
                .default_value("test_")
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        watch: matches.is_present("watch"),
        file_prefix: matches.value_of("file_prefix").unwrap().to_string(),
        test_prefix: matches.value_of("test_prefix").unwrap().to_string(),
    })
}

pub fn run(config: Config) -> Rysult<()> {
    let (tx_files, rx_files) = mpsc::channel();
    let (tx_tests, rx_tests) = mpsc::channel();
    let (tx_results, rx_results) = mpsc::channel();

    let _ = thread::spawn(move || {
        let tx_files = tx_files.clone();
        find_files(config.files.clone(), config.file_prefix.as_str(), config.watch, tx_files).unwrap();
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

pub fn find_files(paths: Vec<String>, prefix: &str, watch: bool, tx: mpsc::Sender<String>) -> Rysult<()> {
    for path in &paths {
        for entry in glob(path.as_str())? {
            match entry {
                Ok(p) => {
                    if p.is_file() && p.file_stem().unwrap().to_string_lossy().starts_with(prefix) && p.extension().unwrap() == "py" {
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
                    Err(e) => println!("Error parsing {}: {}", file_name, e),
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
                let current_dir = env::current_dir().unwrap();
                let path = Path::new(&current_dir);
                let py_code = fs::read_to_string(path.join(test.file.clone()))?;

                let result = Python::with_gil(|py| -> PyResult<Py<PyAny>>{
                    let syspath = py.import_bound("sys").unwrap().getattr("path").unwrap().downcast_into::<PyList>().unwrap();
                    syspath.insert(0, &path).unwrap();

                    let module = PyModule::from_code_bound(py, &py_code, "", "");
                    if module.is_err() {
                        return Err(module.unwrap_err())
                    }
                    
                    let module = module.unwrap();
                    let app = module.getattr(test.test.as_str());
                    if app.is_err() {
                        return Err(app.unwrap_err())
                    }
                    
                    let app: Py<PyAny> = app.unwrap().into();
                    app.call0(py)
                });

                test.passed = result.is_ok();
                tx.send(test)?;
            }, 
            // TODO: Handle this better - should we be able to tell the difference between the channel being closed and an error?
            Err(RecvError) => break,
        }
    }

    Ok(())
}

pub fn output_results(rx: mpsc::Receiver<TestCase>) -> Rysult<()> {
    let mut passed = 0;
    let mut failed = 0;

    loop {
        match rx.recv() {
            Ok(result) => {
                println!("{}:{} - {}", result.file, result.test, if result.passed { "PASSED" } else { "FAILED" });
                if result.passed {
                    passed += 1;
                } else {
                    failed += 1;
                }
            },
            // TODO: Handle this better - should we be able to tell the difference between the channel being closed and an error?
            Err(RecvError) => break,
        }
    }

    println!("{} passed, {} failed", passed, failed);

    Ok(())
}