use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::process::Command;

fn cli() -> Command {
    Command::new(get_cargo_bin("rytest"))
}

fn setup() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    settings.add_filter(r"in [[:xdigit:]]+\.[[:xdigit:]]{2,}s", "in <TIME>s");

    settings
}

#[test]
fn help() {
    let settings = setup();

    settings.bind(|| assert_cmd_snapshot!(cli().arg("--help"), @r###"
                     success: true
                     exit_code: 0
                     ----- stdout -----
                     rytest is a reasonably fast, somewhat Pytest compatible Python test runner.

                     Usage: rytest [OPTIONS] [FILE]...

                     Arguments:
                       [FILE]...  Input file(s) [default: .]

                     Options:
                           --collect-only               only collect tests, don't run them
                       -f, --file-prefix <file_prefix>  The prefix to search for to indicate a file contains tests
                                                        [default: test_]
                       -i, --ignore <ignore>            Ignore file(s) and folders. Can be used multiple times [default:
                                                        .venv]
                           --info                       Print information about rytest and the python environment it is
                                                        running in.
                       -p, --test-prefix <test_prefix>  The prefix to search for to indicate a function is a test
                                                        [default: test_]
                       -v, --verbose                    Verbose output
                       -h, --help                       Print help
                       -V, --version                    Print version

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
        tests/input/good/test_success.py::test_using_fixture
        ERROR tests/input/test_bad_file.py
        tests/input/test_file.py::test_function_passes
        tests/input/test_file.py::test_function_fails
        tests/input/test_file.py::test_parameterized[1]
        tests/input/test_file.py::test_parameterized[2]
        tests/input/test_file.py::test_parameterized[3]
        tests/input/test_file.py::test_parameterized_tuple[1-2]
        tests/input/test_file.py::test_parameterized_tuple[3-4]
        tests/input/test_file.py::test_parameterized_nested[a-1-2]
        tests/input/test_file.py::test_parameterized_nested[a-3-4]
        tests/input/test_file.py::test_parameterized_nested[c-1-2]
        tests/input/test_file.py::test_parameterized_nested[c-3-4]
        tests/input/test_file.py::test_parameterized_expression[0]
        tests/input/test_file.py::test_parameterized_expression[1]
        tests/input/test_file.py::test_parameterized_expression[2]
        tests/input/test_file.py::test_parameterized_functions[round]
        tests/input/test_file.py::test_parameterized_functions[sum]
        tests/input/test_file.py::test_parameterized_functions[int]
        tests/input/test_file.py::test_parameterized_functions[float]
        tests/input/test_fixtures.py::test_fixture
        29 tests collected, 2 errors in <TIME>s

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
fn collect_ignore_folder() {
    let settings = setup();

    settings.bind(|| {
        assert_cmd_snapshot!(cli().arg("tests/input").arg("--collect-only").arg("--ignore").arg("tests/input/bad"), @r###"
        success: true
        exit_code: 0
        ----- stdout -----
        tests/input/classes/test_classes.py::SomeTest::test_something
        tests/input/classes/test_classes.py::SomeTest::test_something_else
        tests/input/classes/test_classes.py::SomeTest::test_assert_failure
        tests/input/folder/test_another_file.py::test_another_function
        tests/input/folder/test_another_file.py::test_function_with_decorator
        tests/input/good/test_success.py::test_success
        tests/input/good/test_success.py::test_more_success
        tests/input/good/test_success.py::test_using_fixture
        ERROR tests/input/test_bad_file.py
        tests/input/test_file.py::test_function_passes
        tests/input/test_file.py::test_function_fails
        tests/input/test_file.py::test_parameterized[1]
        tests/input/test_file.py::test_parameterized[2]
        tests/input/test_file.py::test_parameterized[3]
        tests/input/test_file.py::test_parameterized_tuple[1-2]
        tests/input/test_file.py::test_parameterized_tuple[3-4]
        tests/input/test_file.py::test_parameterized_nested[a-1-2]
        tests/input/test_file.py::test_parameterized_nested[a-3-4]
        tests/input/test_file.py::test_parameterized_nested[c-1-2]
        tests/input/test_file.py::test_parameterized_nested[c-3-4]
        tests/input/test_file.py::test_parameterized_expression[0]
        tests/input/test_file.py::test_parameterized_expression[1]
        tests/input/test_file.py::test_parameterized_expression[2]
        tests/input/test_file.py::test_parameterized_functions[round]
        tests/input/test_file.py::test_parameterized_functions[sum]
        tests/input/test_file.py::test_parameterized_functions[int]
        tests/input/test_file.py::test_parameterized_functions[float]
        tests/input/test_fixtures.py::test_fixture
        27 tests collected, 1 error in <TIME>s

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
        tests/input/good/test_success.py::test_using_fixture
        3 tests collected in <TIME>s

        ----- stderr -----
        "###)
    });
}

#[test]
fn test() {
    let settings = setup();

    settings.bind(|| {
        assert_cmd_snapshot!(cli().arg("tests/input/good").arg("-v"), @r###"
        success: true
        exit_code: 0
        ----- stdout -----
        tests/input/good/test_success.py::test_success - PASSED
        tests/input/good/test_success.py::test_more_success - PASSED
        tests/input/good/test_success.py::test_using_fixture - PASSED
        3 passed, 0 failed in <TIME>s

        ----- stderr -----
        "###)
    });
}
