use anyhow::Result;
use clap::{Arg, Command};

use std::sync::mpsc::{self};
use std::thread;
use std::time::Instant;

mod phases;
mod python;
mod structs;

pub use crate::phases::collection;
pub use crate::phases::execution;
pub use crate::phases::reporting;
pub use crate::structs::{Config, TestCase};

pub fn get_args() -> Result<Config> {
    let matches = Command::new("rytest")
        .version("0.1.0")
        .about("rytest is a reasonably fast, somewhat Pytest compatible Python test runner.")
        // An alphabetical list of arguments
        .arg(
            Arg::new("collect_only")
                .long("collect-only")
                .help("only collect tests, don't run them")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("file_prefix")
                .short('f')
                .long("file-prefix")
                .help("The prefix to search for to indicate a file contains tests")
                .default_value("test_"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value(".")
                .num_args(1..),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .help("Ignore file(s) and folders. Can be used multiple times")
                .default_value(".venv"),
        )
        .arg(
            Arg::new("info")
                .long("info")
                .help("Print information about rytest and the python environment it is running in.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("test_prefix")
                .short('p')
                .long("test-prefix")
                .help("The prefix to search for to indicate a function is a test")
                .default_value("test_"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    Ok(Config {
        collect_only: matches.get_flag("collect_only"),
        file_prefix: matches
            .get_one::<String>("file_prefix")
            .unwrap()
            .to_string(),
        files: matches
            .get_many::<String>("files")
            .unwrap()
            .map(|s| s.to_string())
            .collect(),
        ignores: matches
            .get_many::<String>("ignore")
            .unwrap()
            .map(|s| s.to_string())
            .collect(),
        info: matches.get_flag("info"),
        test_prefix: matches
            .get_one::<String>("test_prefix")
            .unwrap()
            .to_string(),
        verbose: matches.get_flag("verbose"),
    })
}

pub fn run(config: Config) -> Result<()> {
    let start = Instant::now();

    if config.info {
        info()?;
    }

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

fn info() -> Result<()> {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let pyinfo = python::get_info()?;

    println!("Python executable: {:?}", pyinfo.executable);
    println!("Python version: {}", pyinfo.version);
    println!("Python path: {:?}", pyinfo.path);

    Ok(())
}
