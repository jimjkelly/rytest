use clap::{App, Arg};
use pyo3::exceptions::PySyntaxError;
use rustpython_parser::{ast, Parse};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{self};
use std::time::Instant;
use std::{env, fs, thread};

use glob::glob;
use rustpython_parser::ast::Stmt::FunctionDef;

use pyo3::prelude::*;
use pyo3::types::PyList;

type Rysult<T> = Result<T, Box<dyn Error>>;

pub struct Config {
    collect_only: bool,
    files: Vec<String>,
    file_prefix: String,
    test_prefix: String,
    verbose: bool,
}

pub struct TestCase {
    file: String,
    name: String,
    passed: bool,
    error: Option<PyErr>,
}

pub struct Fixture {}

pub fn get_args() -> Rysult<Config> {
    let matches = App::new("rytest")
        .version("0.1.0")
        .author("Jim Kelly <pthread1981@gmail.com>")
        .about("rytest is a reasonably fast, somewhat Pytest compatible Python test runner.")
        // An alphabetical list of arguments
        .arg(
            Arg::with_name("collect_only")
                .long("collect-only")
                .help("only collect tests, don't run them")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("file_prefix")
                .short("f")
                .long("file-prefix")
                .help("The prefix to search for to indicate a file contains tests")
                .default_value("test_"),
        )
        .arg(
            Arg::with_name("test_prefix")
                .short("p")
                .long("test-prefix")
                .help("The prefix to search for to indicate a function is a test")
                .default_value("test_"),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .min_values(1),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Verbose output")
                .takes_value(false),
        )
        .get_matches();

    Ok(Config {
        collect_only: matches.is_present("collect_only"),
        file_prefix: matches.value_of("file_prefix").unwrap().to_string(),
        test_prefix: matches.value_of("test_prefix").unwrap().to_string(),
        files: matches.values_of_lossy("files").unwrap(),
        verbose: matches.is_present("verbose"),
    })
}

pub fn run(config: Config) -> Rysult<()> {
    let start = Instant::now();

    let (tx_files, rx_files) = mpsc::channel();
    let (tx_tests, rx_tests) = mpsc::channel();

    let _ = thread::spawn(move || {
        let tx_files = tx_files.clone();
        find_files(config.files.clone(), config.file_prefix.as_str(), tx_files).unwrap();
    });

    let _ = thread::spawn(move || {
        let tx_tests = tx_tests.clone();
        find_tests(
            config.test_prefix.clone(),
            config.verbose,
            rx_files,
            tx_tests,
        )
        .unwrap();
    });

    if !config.collect_only {
        let (tx_results, rx_results) = mpsc::channel();

        let _ = thread::spawn(move || {
            let tx_results = tx_results.clone();
            run_tests(rx_tests, tx_results).unwrap();
        });

        let handle_output = thread::spawn(move || {
            let rx_results = rx_results;
            output_results(rx_results, start, config.verbose).unwrap();
        });
        handle_output.join().unwrap();
    } else {
        let handle_output = thread::spawn(move || {
            output_collect(rx_tests, start).unwrap();
        });
        handle_output.join().unwrap();
    }

    Ok(())
}

pub fn find_files(paths: Vec<String>, prefix: &str, tx: mpsc::Sender<String>) -> Rysult<()> {
    for path in &paths {
        for entry in glob(path.as_str())? {
            match entry {
                Ok(p) => {
                    if p.is_file()
                        && p.file_stem().unwrap().to_string_lossy().starts_with(prefix)
                        && p.extension().unwrap() == "py"
                    {
                        tx.send(p.to_str().unwrap().to_string())?;
                    }
                }
                Err(e) => println!("Error globbing: {}", e),
            }
        }
    }

    drop(tx);

    Ok(())
}

pub fn find_tests(
    prefix: String,
    verbose: bool,
    rx: mpsc::Receiver<String>,
    tx: mpsc::Sender<TestCase>,
) -> Rysult<()> {
    while let Ok(file_name) = rx.recv() {
        let mut data = String::new();
        let mut file = File::open(file_name.clone())?;
        file.read_to_string(&mut data)?;
        let ast = ast::Suite::parse(data.as_str(), "<embedded>");

        match ast {
            Ok(ast) => {
                for stmt in ast {
                    match stmt {
                        FunctionDef(node) if node.name.starts_with(&prefix) => {
                            let is_pytest_fixture: bool = node.decorator_list.iter()
                            .any(|decorator| {
                                if decorator.is_attribute_expr() {
                                    let attr_expr = decorator.as_attribute_expr().unwrap();
                                    let module = attr_expr.value.as_name_expr().unwrap().id.as_str();
                                    module == "pytest" && attr_expr.attr.as_str() == "fixture"
                                } else {
                                    false
                                }
                            });
                            if !is_pytest_fixture {
                                tx.send(TestCase {
                                    file: file_name.clone(),
                                    name: node.name.to_string(),
                                    passed: false,
                                    error: None,
                                })?
                            }
                        }
                        _ if verbose => println!("{}: Skipping {:?}\n\n", file_name, stmt),
                        _ => {}
                    }
                }
            }
            Err(e) => tx.send(TestCase {
                file: file_name.clone(),
                name: "".to_string(),
                passed: false,
                error: Some(PyErr::new::<PySyntaxError, _>(format!(
                    " Error parsing {}",
                    e
                ))),
            })?,
        }
    }

    Ok(())
}

pub fn run_tests(rx: mpsc::Receiver<TestCase>, tx: mpsc::Sender<TestCase>) -> Rysult<()> {
    while let Ok(mut test) = rx.recv() {
        let currrent_dir = env::current_dir().unwrap();
        let current_dir = Path::new(&currrent_dir);
        let path_buf = current_dir.join(test.file.clone());
        let path = path_buf.as_path();

        let py_code = fs::read_to_string(path)?;

        let result = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
            let syspath = py
                .import_bound("sys")
                .unwrap()
                .getattr("path")
                .unwrap()
                .downcast_into::<PyList>()
                .unwrap();
            syspath.insert(0, path).unwrap();

            let module = PyModule::from_code_bound(py, &py_code, "", "")?;
            let app: Py<PyAny> = module.getattr(test.name.as_str())?.into();
            app.call0(py)
        });

        test.passed = result.is_ok();

        match result.is_ok() {
            true => test.passed = true,
            false => {
                test.error = Some(result.err().unwrap());
                test.passed = false;
            }
        }

        tx.send(test)?;
    }

    Ok(())
}

pub fn output_collect(rx: mpsc::Receiver<TestCase>, start: Instant) -> Rysult<()> {
    let mut collected = 0;
    let mut errors = 0;

    while let Ok(test) = rx.recv() {
        match test.error {
            Some(_) => {
                println!("ERROR {}", test.file);
                errors += 1
            }
            None => {
                println!("{}:{}", test.file, test.name);
                collected += 1;
            }
        }
    }

    let duration = start.elapsed().as_secs_f64();

    match errors {
        0 => println!("{} tests collected in {:.2}s", collected, duration),
        1 => println!(
            "{} tests collected, {} error in {:.2}s",
            collected, errors, duration
        ),
        _ => println!(
            "{} tests collected, {} errors in {:.2}s",
            collected, errors, duration
        ),
    }

    Ok(())
}

pub fn output_results(rx: mpsc::Receiver<TestCase>, start: Instant, verbose: bool) -> Rysult<()> {
    let mut passed = 0;
    let mut failed = 0;

    while let Ok(result) = rx.recv() {
        println!(
            "{}:{} - {}",
            result.file,
            result.name,
            if result.passed { "PASSED" } else { "FAILED" }
        );
        if result.passed {
            passed += 1;
        } else {
            failed += 1;
            if verbose {
                if let Some(error) = result.error {
                    println!("{}", error);
                }
            }
        }
    }

    let duration = start.elapsed().as_secs_f64();

    println!("{} passed, {} failed in {:2}s", passed, failed, duration);

    Ok(())
}
