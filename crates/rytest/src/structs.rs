use pyo3::PyErr;

#[derive(Debug)]
pub struct Config {
    pub collect_only: bool,
    pub file_prefix: String,
    pub files: Vec<String>,
    pub ignores: Vec<String>,
    pub info: bool,
    pub test_prefix: String,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct TestCase {
    pub file: String,
    pub name: String,
    pub passed: bool,
    pub error: Option<PyErr>,
    pub parametrized: bool,
}

#[derive(Debug)]
pub struct PyInfo {
    pub executable: String,
    pub version: String,
    pub path: Vec<String>,
}
