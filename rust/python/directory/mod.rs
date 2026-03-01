use pyo3::prelude::*;
use pyo3::types::PyList;

use std::path::PathBuf;
use std::sync::Arc;

use ::organizeit2::PathLike;

use ::organizeit2::Directory as BaseDirectory;

use super::entry_to_pyobject;
use super::PyFsspecFileSystem;

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct Directory {
    pub inner: BaseDirectory,
}

#[pymethods]
impl Directory {
    #[new]
    #[pyo3(signature = (path))]
    fn new(path: String) -> PyResult<Self> {
        match BaseDirectory::new(&path) {
            Ok(inner) => Ok(Directory { inner }),
            Err(_) => {
                // Fallback: try Python fsspec for unknown protocols
                let (fs, stripped) = PyFsspecFileSystem::from_url(&path)?;
                let filesystem = Arc::new(fs) as Arc<dyn fsspec_rs::FileSystem + Send + Sync>;
                let inner = BaseDirectory {
                    path: PathBuf::from(&stripped),
                    filesystem,
                };
                Ok(Directory { inner })
            }
        }
    }

    fn __str__(&self) -> String {
        self.inner.display_path()
    }

    fn __repr__(&self) -> String {
        self.inner.repr()
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner.display_path().hash(&mut hasher);
        hasher.finish()
    }

    fn __lt__(&self, other: &Directory) -> bool {
        self.inner.display_path() < other.inner.display_path()
    }

    fn __eq__(&self, other: &Directory) -> bool {
        self.inner == other.inner
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn str(&self) -> String {
        self.__str__()
    }

    fn exists(&self) -> bool {
        self.inner.exists()
    }

    fn as_posix(&self) -> String {
        self.inner.as_posix()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name()
    }

    #[getter]
    fn suffix(&self) -> String {
        self.inner.suffix()
    }

    #[getter]
    fn stem(&self) -> String {
        self.inner.stem()
    }

    #[getter]
    fn parts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let parts = self.inner.parts();
        PyList::new(py, parts)
    }

    #[getter]
    fn parent(&self) -> Directory {
        Directory {
            inner: self.inner.parent(),
        }
    }

    fn modified<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let time = self
            .inner
            .modified()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyOSError, _>(e.to_string()))?;
        let duration = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let timestamp = duration.as_secs_f64();
        let datetime = py.import("datetime")?.getattr("datetime")?;
        datetime.call_method1("fromtimestamp", (timestamp,))
    }

    fn resolve<'py>(&self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        entry_to_pyobject(py, self.inner.resolve())
    }

    #[pyo3(name = "match", signature = (pattern, *, name_only=true, invert=false))]
    fn match_glob(&self, pattern: String, name_only: bool, invert: bool) -> bool {
        self.inner.match_glob(&pattern, name_only, invert)
    }

    #[pyo3(signature = (pattern, *, name_only=true, invert=false))]
    fn all_match<'py>(
        &self,
        py: Python<'py>,
        pattern: String,
        name_only: bool,
        invert: bool,
    ) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .all_match(&pattern, name_only, invert)
            .into_iter()
            .map(|e| entry_to_pyobject(py, e))
            .collect()
    }

    #[pyo3(signature = (re, *, name_only=true, invert=false))]
    fn rematch(&self, re: String, name_only: bool, invert: bool) -> bool {
        self.inner.match_re(&re, name_only, invert)
    }

    #[pyo3(signature = (re, *, name_only=true, invert=false))]
    fn all_rematch<'py>(
        &self,
        py: Python<'py>,
        re: String,
        name_only: bool,
        invert: bool,
    ) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .all_rematch(&re, name_only, invert)
            .into_iter()
            .map(|e| entry_to_pyobject(py, e))
            .collect()
    }

    #[pyo3(signature = (other, *, soft=true))]
    fn link(&self, other: &Directory, soft: bool) -> PyResult<()> {
        self.inner
            .link_to(&other.inner.path, soft)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
    }

    fn unlink(&self) -> PyResult<()> {
        self.inner
            .unlink()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyOSError, _>(e.to_string()))
    }

    fn rm(&self) -> PyResult<()> {
        self.inner
            .rm()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyOSError, _>(e.to_string()))
    }

    fn ls<'py>(&self, py: Python<'py>) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .ls()
            .into_iter()
            .map(|e| entry_to_pyobject(py, e))
            .collect()
    }

    fn list(&self) -> Vec<String> {
        self.inner.list()
    }

    fn recurse<'py>(&self, py: Python<'py>) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .recurse()
            .into_iter()
            .map(|e| entry_to_pyobject(py, e))
            .collect()
    }

    #[pyo3(signature = (block_size=4096))]
    fn size(&self, block_size: u64) -> u64 {
        self.inner.size(block_size)
    }

    fn __truediv__<'py>(&self, py: Python<'py>, other: String) -> PyResult<Py<PyAny>> {
        entry_to_pyobject(py, self.inner.join(&other))
    }
}
