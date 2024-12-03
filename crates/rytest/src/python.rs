use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::env;
use std::path::Path;

use crate::structs::PyInfo;

pub fn setup<'a>(py: &'a Python, current_dir: &Path) -> Bound<'a, PyList> {
    let syspath = py
        .import_bound("sys")
        .unwrap()
        .getattr("path")
        .unwrap()
        .downcast_into::<PyList>()
        .unwrap();

    syspath.insert(0, current_dir).unwrap();

    if let Ok(venv) = env::var("VIRTUAL_ENV") {
        let venv_path = Path::new(&venv);
        let version = py.version_info();
        let site_packages = venv_path.join(format!(
            "lib/python{}.{}/site-packages",
            version.major, version.minor
        ));
        syspath.insert(0, site_packages).unwrap();
    }

    syspath
}

pub fn get_info() -> Result<PyInfo> {
    Python::with_gil(|py| -> Result<PyInfo> {
        let syspath = setup(&py, std::env::current_dir().unwrap().as_path());

        let sys = py.import_bound("sys")?;

        let version: String = sys.getattr("version").unwrap().extract()?;

        let executable: String = sys.getattr("executable").unwrap().extract()?;

        Ok(PyInfo {
            executable,
            version,
            path: syspath.extract()?,
        })
    })
}
