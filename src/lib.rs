use anyhow::Result;
use clap::{App, Arg};

use std::sync::mpsc::{self};
use std::thread;
use std::time::Instant;

mod phases;
mod structs;

pub use crate::phases::collection;
pub use crate::phases::execution;
pub use crate::phases::reporting;
pub use crate::structs::{Config, TestCase};

pub fn get_args() -> Result<Config> {
    let matches = App::new("rytest")
        .version("0.1.0")
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
            Arg::with_name("ignore")
                .short("i")
                .long("ignore")
                .help("Ignore file(s) and folders. Can be used multiple times")
                .default_value(".venv"),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value(".")
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
        ignores: matches.values_of_lossy("ignore").unwrap(),
        verbose: matches.is_present("verbose"),
    })
}

pub fn run(config: Config) -> Result<()> {
    let start = Instant::now();

    let (tx_files, rx_files) = mpsc::channel();
    let (tx_tests, rx_tests) = mpsc::channel();

    let _ = thread::spawn(move || {
        let tx_files = tx_files.clone();
        collection::find_files(
            config.files,
            config.ignores,
            config.file_prefix.as_str(),
            tx_files,
        )
        .unwrap();
    });

    let _ = thread::spawn(move || {
        let tx_tests = tx_tests.clone();
        collection::find_tests(
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
            execution::run_tests(rx_tests, tx_results).unwrap();
        });

        let handle_output = thread::spawn(move || {
            let rx_results = rx_results;
            reporting::output_results(rx_results, start, config.verbose).unwrap();
        });
        handle_output.join().unwrap();
    } else {
        let handle_output = thread::spawn(move || {
            reporting::output_collect(rx_tests, start, config.verbose).unwrap();
        });
        handle_output.join().unwrap();
    }

    Ok(())
}
