use pyo3::PyErr;

pub struct Config {
    pub collect_only: bool,
    pub files: Vec<String>,
    pub file_prefix: String,
    pub test_prefix: String,
    pub verbose: bool,
}

pub struct TestCase {
    pub file: String,
    pub name: String,
    pub passed: bool,
    pub error: Option<PyErr>,
}
