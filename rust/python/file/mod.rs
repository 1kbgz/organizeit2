use pyo3::prelude::*;
use pyo3::types::PyList;

use std::path::PathBuf;
use std::sync::Arc;

use ::organizeit2::PathLike;

use ::organizeit2::File as BaseFile;

use super::entry_to_pyobject;
use super::PyFsspecFileSystem;

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct File {
    pub inner: BaseFile,
}

#[pymethods]
impl File {
    #[new]
    #[pyo3(signature = (path))]
    fn new(path: String) -> PyResult<Self> {
        match BaseFile::new(&path) {
            Ok(inner) => Ok(File { inner }),
            Err(_) => {
                // Fallback: try Python fsspec for unknown protocols
                let (fs, stripped) = PyFsspecFileSystem::from_url(&path)?;
                let filesystem = Arc::new(fs) as Arc<dyn fsspec_rs::FileSystem + Send + Sync>;
                let inner = BaseFile {
                    path: PathBuf::from(&stripped),
                    filesystem,
                };
                Ok(File { inner })
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

    fn __lt__(&self, other: &File) -> bool {
        self.inner.display_path() < other.inner.display_path()
    }

    fn __eq__(&self, other: &File) -> bool {
        self.inner == other.inner
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
    fn parent(&self) -> super::directory::Directory {
        super::directory::Directory {
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

    #[pyo3(signature = (re, *, name_only=true, invert=false))]
    fn rematch(&self, re: String, name_only: bool, invert: bool) -> bool {
        self.inner.match_re(&re, name_only, invert)
    }

    #[pyo3(signature = (other, *, soft=true))]
    fn link(&self, other: &File, soft: bool) -> PyResult<()> {
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

    #[pyo3(signature = (block_size=4096))]
    fn size(&self, block_size: u64) -> PyResult<u64> {
        self.inner
            .size(block_size)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyOSError, _>("could not determine file size"))
    }

    fn __truediv__<'py>(&self, py: Python<'py>, other: String) -> PyResult<Py<PyAny>> {
        entry_to_pyobject(py, self.inner.join(&other))
    }
}
