use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::process::Command;

fn cli() -> Command {
    Command::new(get_cargo_bin("rytest"))
}

fn setup() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    settings.add_filter(r"in [[:xdigit:]]+\.[[:xdigit:]]{2}s", "in <TIME>s");

    settings
}

#[test]
fn help() {
    let settings = setup();

    settings.bind(|| assert_cmd_snapshot!(cli().arg("--help"), @r###"
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
        <FILE>...    Input file(s) [default: .]

    ----- stderr -----
    "###));
}

#[test]
fn collect_errors() {
    let settings = setup();

    settings.bind(|| {
        assert_cmd_snapshot!(cli().arg("tests").arg("--collect-only"), @r###"
                     success: true
                     exit_code: 0
                     ----- stdout -----
                     tests/input/classes/test_classes.py::SomeTest::test_something
                     tests/input/classes/test_classes.py::SomeTest::test_something_else
                     tests/input/classes/test_classes.py::SomeTest::test_assert_failure
                     ERROR tests/input/bad/test_other_error.py
                     tests/input/bad/test_other_file.py::test_function_passes
                     tests/input/bad/test_other_file.py::test_function_fails
                     tests/input/folder/test_another_file.py::test_another_function
                     tests/input/folder/test_another_file.py::test_function_with_decorator
                     ERROR tests/input/test_bad_file.py
                     tests/input/good/test_success.py::test_success
                     tests/input/good/test_success.py::test_more_success
                     tests/input/test_file.py::test_function_passes
                     tests/input/test_file.py::test_function_fails
                     tests/input/test_fixtures.py::test_fixture
                     12 tests collected, 2 errors in <TIME>s

                     ----- stderr -----
                     "###)
    });
}

#[test]
fn collect_error() {
    let settings = setup();

    settings.bind(|| {
        assert_cmd_snapshot!(cli().arg("tests/input/bad").arg("--collect-only"), @r###"
                     success: true
                     exit_code: 0
                     ----- stdout -----
                     ERROR tests/input/bad/test_other_error.py
                     tests/input/bad/test_other_file.py::test_function_passes
                     tests/input/bad/test_other_file.py::test_function_fails
                     2 tests collected, 1 error in <TIME>s

                     ----- stderr -----
                     "###)
    });
}

#[test]
fn collect() {
    let settings = setup();

    settings.bind(|| {
        assert_cmd_snapshot!(cli().arg("tests/input/good").arg("--collect-only"), @r###"
                     success: true
                     exit_code: 0
                     ----- stdout -----
                     tests/input/good/test_success.py::test_success
                     tests/input/good/test_success.py::test_more_success
                     2 tests collected in <TIME>s

                     ----- stderr -----
                     "###)
    });
}
