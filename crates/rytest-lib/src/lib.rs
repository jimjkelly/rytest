use pyo3::{prelude::*, types::{PyList, PyTuple}};


#[derive(FromPyObject)]
struct Session {
    items: Py<PyList>,
    testscollected: i32,
}

#[pyclass]
struct Item {
    path: String,
    name: String,
}

#[pymethods]
impl Item {
    pub fn runtest(&self) {}

    pub fn reportinfo(&self) -> Py<PyTuple> {
        Python::with_gil(|py| {
            let tuple = (self.path.clone(), 0, format!("custom test: {}", self.name));
            tuple.into_py(py)
        })
    }


}

#[pyfunction]
fn pytest_collection(mut session: Session) -> PyResult<bool> {
    Python::with_gil(|py| {
        let items = session.items.into_bound(py);
        items.append(Item{path: "Hello".to_string(), name: "World".to_string()}.into_py(py)).unwrap();
        session.testscollected = 42;
    });

    Ok(true)
}

#[pymodule]
fn rytest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pytest_collection, m)?)?;
    Ok(())
}