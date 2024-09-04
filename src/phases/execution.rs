use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::{env, fs, path::Path, sync::mpsc};

use crate::TestCase;

pub fn run_tests(rx: mpsc::Receiver<TestCase>, tx: mpsc::Sender<TestCase>) -> Result<()> {
    while let Ok(mut test) = rx.recv() {
        let currrent_dir = env::current_dir().unwrap();
        let current_dir = Path::new(&currrent_dir);
        let path_buf = current_dir.join(test.file.clone());
        let path = path_buf.as_path();

        let py_code = fs::read_to_string(path)?;

        let result = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
            let syspath = py
                .import_bound("sys")
                .unwrap()
                .getattr("path")
                .unwrap()
                .downcast_into::<PyList>()
                .unwrap();

            syspath.insert(0, path).unwrap();

            let module = PyModule::from_code_bound(py, &py_code, "", "")?;
            let function: Py<PyAny> = module.getattr(test.name.as_str())?.into();
            let mut result = function.call0(py);
            let py_list: &PyList = result.as_mut().unwrap().extract(py)?;
            println!("{}: {:#?}", test.name, py_list);
            result

        });

        test.passed = result.is_ok();

        match result.is_ok() {
            true => test.passed = true,
            false => {
                test.error = Some(result.err().unwrap());
                test.passed = false;
            }
        }

        tx.send(test)?;
    }

    Ok(())
}
