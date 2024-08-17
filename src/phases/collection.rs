use anyhow::Result;
use glob::glob;
use pyo3::exceptions::PySyntaxError;
use pyo3::PyErr;
use rustpython_parser::ast::Stmt::FunctionDef;
use rustpython_parser::{ast, Parse};
use std::io::Read;
use std::{fs::File, sync::mpsc};

use crate::TestCase;

use crate::phases::collectors::ignore_test;

pub fn find_files(paths: Vec<String>, prefix: &str, tx: mpsc::Sender<String>) -> Result<()> {
    for path in &paths {
        for entry in glob(path.as_str())? {
            match entry {
                Ok(p) => {
                    if p.is_file()
                        && p.file_stem().unwrap().to_string_lossy().starts_with(prefix)
                        && p.extension().unwrap() == "py"
                    {
                        tx.send(p.to_str().unwrap().to_string())?;
                    }
                }
                Err(e) => println!("Error globbing: {}", e),
            }
        }
    }

    drop(tx);

    Ok(())
}

pub fn find_tests(
    prefix: String,
    verbose: bool,
    rx: mpsc::Receiver<String>,
    tx: mpsc::Sender<TestCase>,
) -> Result<()> {
    while let Ok(file_name) = rx.recv() {
        let mut data = String::new();
        let mut file = File::open(file_name.clone())?;
        file.read_to_string(&mut data)?;
        let ast = ast::Suite::parse(data.as_str(), "<embedded>");

        match ast {
            Ok(ast) => {
                for stmt in ast {
                    match stmt {
                        FunctionDef(ref node) if node.name.starts_with(&prefix) => {
                            let is_pytest_fixture: bool =
                                ignore_test::is_pytest_fixture(stmt.clone());
                            if !is_pytest_fixture {
                                tx.send(TestCase {
                                    file: file_name.clone(),
                                    name: node.name.to_string(),
                                    passed: false,
                                    error: None,
                                })?
                            }
                        }
                        _ if verbose => println!("{}: Skipping {:?}\n\n", file_name, stmt),
                        _ => {}
                    }
                }
            }
            Err(e) => tx.send(TestCase {
                file: file_name.clone(),
                name: "".to_string(),
                passed: false,
                error: Some(PyErr::new::<PySyntaxError, _>(format!(
                    " Error parsing {}",
                    e
                ))),
            })?,
        }
    }

    Ok(())
}
