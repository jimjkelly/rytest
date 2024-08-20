use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::process::Command;

fn cli() -> Command {
    Command::new(get_cargo_bin("rytest"))
}

#[test]
fn help() {
    assert_cmd_snapshot!(cli().arg("--help"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    rytest 0.1.0
    rytest is a reasonably fast, somewhat Pytest compatible Python test runner.

    USAGE:
        rytest [FLAGS] [OPTIONS] [FILE]...

    FLAGS:
            --collect-only    only collect tests, don't run them
        -h, --help            Prints help information
        -V, --version         Prints version information
        -v, --verbose         Verbose output

    OPTIONS:
        -f, --file-prefix <file_prefix>    The prefix to search for to indicate a file contains tests [default: test_]
        -p, --test-prefix <test_prefix>    The prefix to search for to indicate a function is a test [default: test_]

    ARGS:
        <FILE>...    Input file(s) [default: -]

    ----- stderr -----
    "###);
}

#[test]
fn collect_errors() {
    assert_cmd_snapshot!(cli().arg("tests/**/*.py").arg("--collect-only"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    ERROR tests/input/bad/test_other_error.py
    tests/input/bad/test_other_file.py::test_function_passes
    tests/input/bad/test_other_file.py::test_function_fails
    tests/input/classes/test_classes.py::SomeTest::test_something
    tests/input/classes/test_classes.py::SomeTest::test_something_else
    tests/input/classes/test_classes.py::SomeTest::test_assert_failure
    tests/input/folder/test_another_file.py::test_another_function
    tests/input/folder/test_another_file.py::test_function_with_decorator
    tests/input/good/test_success.py::test_success
    tests/input/good/test_success.py::test_more_success
    ERROR tests/input/test_bad_file.py
    tests/input/test_file.py::test_function_passes
    tests/input/test_file.py::test_function_fails
    tests/input/test_fixtures.py::test_fixture
    12 tests collected, 2 errors in 0.00s

    ----- stderr -----
    "###);
}

#[test]
fn collect_error() {
    assert_cmd_snapshot!(cli().arg("tests/input/bad/*.py").arg("--collect-only"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    ERROR tests/input/bad/test_other_error.py
    tests/input/bad/test_other_file.py::test_function_passes
    tests/input/bad/test_other_file.py::test_function_fails
    2 tests collected, 1 error in 0.00s

    ----- stderr -----
    "###);
}

#[test]
fn collect() {
    assert_cmd_snapshot!(cli().arg("tests/input/good/*.py").arg("--collect-only"), @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    tests/input/good/test_success.py::test_success
    tests/input/good/test_success.py::test_more_success
    2 tests collected in 0.00s

    ----- stderr -----
    "###);
}
