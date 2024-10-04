use anyhow::Result;
use std::{sync::mpsc, time::Instant};

use crate::TestCase;

pub fn output_collect(rx: mpsc::Receiver<TestCase>, start: Instant, verbose: bool) -> Result<()> {
    let mut collected = 0;
    let mut errors = 0;

    while let Ok(test) = rx.recv() {
        match test.error {
            Some(_) => {
                if test.name.is_empty() {
                    println!("ERROR {}", test.file);
                } else {
                    println!("ERROR {}::{}", test.file, test.name);
                }
                if verbose {
                    if let Some(error) = test.error {
                        println!("{}", error);
                    }
                }
                errors += 1
            }
            None => {
                println!("{}::{}", test.file, test.name);
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

pub fn output_results(rx: mpsc::Receiver<TestCase>, start: Instant, verbose: bool) -> Result<()> {
    let mut passed = 0;
    let mut failed = 0;

    while let Ok(result) = rx.recv() {
        println!(
            "{}::{} - {}",
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
