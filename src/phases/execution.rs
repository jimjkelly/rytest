use anyhow::Result;
use pyo3::indoc::indoc;
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
            function.call0(py)
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

pub fn get_parametrizations(path: &str, name: &str) -> Result<Vec<String>, PyErr> {
    let currrent_dir = env::current_dir().unwrap();
    let current_dir = Path::new(&currrent_dir);
    let path_buf = current_dir.join(path);
    let path = path_buf.as_path();

    let mut py_code = fs::read_to_string(path)?;
    let s1 = indoc! {"
    import pytest
    import itertools
    def get_parameter_name(obj):
        if isinstance(obj, list) or isinstance(obj, tuple):
            return '-'.join([get_parameter_name(o) for o in obj])
        
        
        if hasattr(obj, '__name__'):
            return obj.__name__
        else:
            return str(obj)

    def decorator_factory(argnames, argvalues):
        def decorator(function):
            # Generate all parameter combinations if multiple decorators are used
            if not hasattr(function, 'parameters'):
                function.parameters = []

            parameters = [get_parameter_name(v) for v in argvalues]
            if function.parameters:
                parameters = list(itertools.product(parameters, function.parameters))
            setattr(function, 'parameters', [get_parameter_name(v) for v in parameters])
            
            return function
        return decorator
    

    pytest.mark.parametrize =  decorator_factory

    "};
    py_code.insert_str(0, s1);

    let result = Python::with_gil(|py| -> PyResult<Vec<String>> {
        let syspath = py
            .import_bound("sys")
            .unwrap()
            .getattr("path")
            .unwrap()
            .downcast_into::<PyList>()
            .unwrap();

        let venv = env::var("VIRTUAL_ENV");
        if venv.is_ok() {
            let venv = venv.unwrap();
            let venv_path = Path::new(&venv);
            let version = py.version_info();
            let site_packages = venv_path.join(format!("lib/python{}.{}/site-packages", version.major, version.minor));
            syspath.insert(0, site_packages).unwrap();
        }

        syspath.insert(0, current_dir).unwrap();
        syspath.insert(0, path).unwrap();

        let module = PyModule::from_code_bound(py, &py_code, "", "");
        if module.is_err() {
            return Err(module.err().unwrap());
        }
        let function_instance = module.unwrap().clone().getattr(name);
        if function_instance.is_err() {
            return Err(function_instance.err().unwrap());
        }
        let function: Py<PyAny> = function_instance?.into();
        function.getattr(py, "parameters").unwrap().extract(py)
    });
    result
}
