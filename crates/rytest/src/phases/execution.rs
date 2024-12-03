use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyMapping, PyString};
use pyo3::{indoc::indoc, types::PyTuple};
use std::{env, fs, path::Path, sync::mpsc};

use crate::python;
use crate::TestCase;

pub fn run_tests(rx: mpsc::Receiver<TestCase>, tx: mpsc::Sender<TestCase>) -> Result<()> {
    while let Ok(mut test) = rx.recv() {
        if test.parametrized {
            // skip parametrized function since they are not supported yet
            test.passed = false;
            test.error = Some(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                "Parametrized tests are not supported yet".to_string(),
            ));
            tx.send(test)?;
            continue;
        }
        let currrent_dir = env::current_dir().unwrap();
        let current_dir = Path::new(&currrent_dir);
        let path_buf = current_dir.join(test.file.clone());
        let path = path_buf.as_path();

        let mut py_code = fs::read_to_string(path)?;
        // replace pytest fixture with noop so we can call it directly
        let s1 = indoc! {"
        import pytest
        pytest.fixture = lambda func: func
        "};
        py_code.insert_str(0, s1);

        let result = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
            let syspath = python::setup(&py, current_dir);

            syspath.insert(0, path).unwrap();

            let module = PyModule::from_code_bound(py, &py_code, "", "")?;
            let function: Py<PyAny> = module.getattr(test.name.as_str())?.into();

            let inspect = py.import_bound("inspect")?;
            let signature = inspect
                .getattr("signature")?
                .call1((module.getattr(test.name.as_str())?,))?;
            let binding = signature.getattr("parameters")?;
            let parameters = binding.downcast::<PyMapping>()?;

            // Prepare a vector to hold the positional arguments
            let mut args_vec: Vec<PyObject> = Vec::new();
            // Prepare a vector to hold the generators to run after the fixture is called
            let mut generators: Vec<Py<PyAny>> = Vec::new();

            for item in parameters.items()?.iter()? {
                let item = item?;
                let param_name_obj = item.get_item(0)?; // First item is the parameter name
                let param_name: String = param_name_obj.extract()?;
                let param_name_py = PyString::new_bound(py, &param_name);
                // Check if the module has a function with the same name as the parameter
                if let Ok(func) = module.getattr(param_name_py) {
                    // If a matching function is found, call it and store the result in args_vec
                    let value: PyObject = func.call0()?.into();
                    let value_iter: Result<Py<PyIterator>, PyErr> = value.extract(py);
                    if value_iter.is_ok() {
                        // call next on the iterator to get the value
                        if let Ok(iterator) = value.getattr(py, "__iter__")?.call0(py) {
                            // Attempt to call __next__ to get the actual value from the generator/iterator
                            match iterator.getattr(py, "__next__")?.call0(py) {
                                Ok(next_value) => {
                                    args_vec.push(next_value);
                                    generators.push(iterator);
                                }
                                Err(err) => {
                                    return Err(err);
                                }
                            }
                        }
                    } else {
                        // If __iter__ doesn't exist, use the value directly
                        args_vec.push(value);
                    }
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "No matching function found for parameter: {}",
                        param_name
                    )));
                }
            }

            // Create a PyTuple from the arguments vector
            let args_tuple = PyTuple::new_bound(py, &args_vec);

            let test_result = function.call1(py, args_tuple);
            // Execute remaining generator items (optional)
            for generator in generators {
                while let Ok(_next_item) = generator.getattr(py, "__next__")?.call0(py) {
                    // just eat the result
                }
            }
            test_result
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
        let syspath = python::setup(&py, current_dir);

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
