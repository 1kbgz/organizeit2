use pyo3::prelude::*;

pub mod directory;
pub mod file;
pub mod organizeit;

pub use directory::Directory;
pub use file::File;
pub use organizeit::OrganizeIt;

/// Convert a Rust Entry to the appropriate Python object.
pub fn entry_to_pyobject(py: Python, entry: ::organizeit2::Entry) -> PyResult<Py<PyAny>> {
    match entry {
        ::organizeit2::Entry::File(f) => Ok(Py::new(py, File { inner: f })?.into_any()),
        ::organizeit2::Entry::Directory(d) => Ok(Py::new(py, Directory { inner: d })?.into_any()),
    }
}

/// Factory function: resolve a path string to a File or Directory.
#[pyfunction]
#[pyo3(name = "Path")]
fn resolve_path(py: Python, path: String) -> PyResult<Py<PyAny>> {
    let entry = ::organizeit2::resolve_path(&path);
    entry_to_pyobject(py, entry)
}

#[pymodule]
fn organizeit2(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Directory>()?;
    m.add_class::<File>()?;
    m.add_class::<OrganizeIt>()?;
    m.add_function(wrap_pyfunction!(resolve_path, m)?)?;
    Ok(())
}
