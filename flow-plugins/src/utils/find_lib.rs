#[cfg(feature = "python")]
pub fn find_lib_from_python(module_name: &str, libs: &[&str]) -> Option<Vec<std::path::PathBuf>> {
    use pyo3::prelude::*;
    if let Ok(path) = Python::with_gil(|py| -> PyResult<String> {
        let module = py.import(module_name)?;
        module.getattr("__file__")?.extract()
    }) {
        let mut path = std::path::PathBuf::from(path);
        path.pop();
        Some(
            libs.iter()
                .map(|x| {
                    let mut path = path.clone();
                    path.push(x);
                    path
                })
                .collect(),
        )
    } else {
        None
    }
}
