use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyMapping};
use pyo3::{indoc::indoc, types::PyTuple};
use std::path::PathBuf;
use std::{env, fs, path::Path, sync::mpsc};

use crate::python;
use crate::TestCase;

fn file_contains_fixture(file_path: &Path, fixture_name: &str) -> bool {
    if let Ok(content) = fs::read_to_string(file_path) {
        let def_pattern = format!("def {}(", fixture_name);
        let fixture_pattern = format!("fixture(name=\"{}\")", fixture_name);
        return content.contains(&def_pattern) || content.contains(&fixture_pattern);
    }
    false
}

fn find_fixture_reversely(
    start_path: &Path,
    fixture_name: &str,
    stop_path: &PathBuf,
) -> Option<PathBuf> {
    let mut current_path = start_path.to_path_buf();

    // Check if start_path is a file and search it first
    if current_path.is_file() {
        if file_contains_fixture(&current_path, fixture_name) {
            return Some(current_path);
        }
        // Move to the parent directory if the start path was a file
        current_path.pop();
    }

    // Begin directory traversal
    loop {
        //n!("Checking directory for fixture {}: {:?}", fixture_name, current_path);

        if let Ok(entries) = fs::read_dir(&current_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    continue; // Skip directories
                }

                // Check if the file contains the fixture
                if path.extension().map_or(false, |ext| ext == "py")
                    && file_contains_fixture(&path, fixture_name)
                {
                    return Some(path);
                }
            }
        }

        // Stop if we have reached the stopping path
        if current_path == *stop_path {
            break;
        }

        // Move up one directory level
        if !current_path.pop() {
            break; // Stop if we've reached the root
        }
    }

    None
}

fn run_fixture(path: &Path, fixture_name: &str, py: Python) -> Result<PyObject, PyErr> {
    let currrent_dir = env::current_dir().unwrap();
    let current_dir = Path::new(&currrent_dir);
    let path_buf = current_dir.join(path);
    let path = path_buf.as_path();

    let found = find_fixture_reversely(path, fixture_name, &currrent_dir);
    match found {
        Some(file) => {
            //println!("Found fixture in file: {:?}", file);
            let mut py_code = fs::read_to_string(file)?;
            // replace pytest fixture with noop so we can call it directly
            let s1 = indoc! {"
            import pytest
            pytest.fixture = lambda func: func
            "};
            py_code.insert_str(0, s1);

            let module = PyModule::from_code_bound(py, &py_code, "", "")?;
            let function: Py<PyAny> = module.getattr(fixture_name)?.into();
            let value: PyObject = function.call0(py)?;
            Ok(value)
        }
        None => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "No matching function found for fixture: {}",
            fixture_name
        ))),
    }
}

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

            //println!("File path, current_dir: {:?}, {:?}", path, current_dir);

            for item in parameters.items()?.iter()? {
                let item = item?;
                let param_name_obj = item.get_item(0)?; // First item is the parameter name
                let param_name: String = param_name_obj.extract()?;

                let res = run_fixture(path, param_name.as_str(), py);

                match res {
                    Ok(value) => {
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
                    }
                    Err(err) => {
                        return Err(err);
                    }
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
