use assert_cmd::Command;
use std::fs;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const PRG: &str = "rytest";

fn run(args: &[&str], expected_file: &str) -> TestResult {
    println!("expected {}", &expected_file);
    let expected = fs::read_to_string(expected_file)?;
    Command::cargo_bin(PRG)?
        .args(args)
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

#[test]
fn help() -> TestResult {
    run(&["--help"], "tests/expected/help.out")
}

#[test]
fn collect_errors() -> TestResult {
    run(
        &["tests/**/*.py", "--collect-only"],
        "tests/expected/collect_two_errors.out",
    )
}

#[test]
fn collect_error() -> TestResult {
    run(
        &["tests/input/bad/*.py", "--collect-only"],
        "tests/expected/collect_one_error.out",
    )
}

#[test]
fn collect() -> TestResult {
    run(
        &["tests/input/good/*.py", "--collect-only"],
        "tests/expected/collect.out",
    )
}
