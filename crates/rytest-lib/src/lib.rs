use pyo3::{intern, prelude::*, types::{PyBool, PyFunction, PyString}};
use walkdir::WalkDir;


#[pyclass]
struct Collector {
    path: String,
    config: PyObject,
    ignore_collect: Py<PyFunction>,
    collect_file: Py<PyFunction>,
    parent: PyObject,
}

#[pymethods]
impl Collector {
    #[new]
    fn new(path: String, config: PyObject, ignore_collect: Py<PyFunction>, collect_file: Py<PyFunction>, parent: PyObject) -> Self {
        Collector {
            path: path,
            config: config,
            ignore_collect: ignore_collect,
            collect_file: collect_file,
            parent: parent,
        }
    }

    fn collect(&self) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            let pathlib = PyModule::import_bound(py, intern!(py, "pathlib"))?;
            let path = pathlib.getattr(intern!(py, "Path"))?;
            
            let tests = WalkDir::new(self.path.clone())
                .sort_by_file_name()
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| {
                    let os_path = e.path().as_os_str();
                    let str_path = os_path.to_str().unwrap();
                    let pystr_path = PyString::new_bound(py, str_path);
                    let bound_path = path.call1((pystr_path, )).unwrap();
                    let path = bound_path.unbind();
                    path
                })
                .filter(|path| {
                    return true;
                    
                    // Trying to figure this out:
                    // https://docs.pytest.org/en/7.1.x/reference/reference.html#pytest.hookspec.pytest_ignore_collect
                    match self.ignore_collect.call1(py, (path, self.config.clone_ref(py))) {
                        Ok(r) if !r.is_none(py) => r.extract::<bool>(py).unwrap(),
                        Ok(_) => false,
                        Err(_) => false
                    }
                })
                .map(|path| self.collect_file.call1(py, (path, self.parent.clone_ref(py))).unwrap())
                .collect();

            Ok(tests)
        })
    }
}


#[pymodule]
fn rytest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Collector>()?;
    Ok(())
}
