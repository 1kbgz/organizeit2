use std::collections::HashMap;

use fsspec_rs::{
    FileInfo, FileSystem, FileType, FsError, FsFile, FsResult, OpenMode, OpenOptions,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;

// =========================================================================
// Helper: convert a PyErr into an FsError
// =========================================================================

fn py_err_to_fs(e: PyErr) -> FsError {
    Python::attach(|py| {
        let msg = e.to_string();
        if e.is_instance_of::<pyo3::exceptions::PyFileNotFoundError>(py) {
            FsError::NotFound(msg)
        } else if e.is_instance_of::<pyo3::exceptions::PyPermissionError>(py) {
            FsError::PermissionDenied(msg)
        } else if e.is_instance_of::<pyo3::exceptions::PyFileExistsError>(py) {
            FsError::AlreadyExists(msg)
        } else if e.is_instance_of::<pyo3::exceptions::PyNotADirectoryError>(py) {
            FsError::NotADirectory(msg)
        } else if e.is_instance_of::<pyo3::exceptions::PyIsADirectoryError>(py) {
            FsError::IsADirectory(msg)
        } else {
            FsError::Other(msg)
        }
    })
}

// =========================================================================
// PyFsspecFileSystem: wraps a Python fsspec filesystem object
// =========================================================================

/// A [`FileSystem`] implementation that delegates to a Python `fsspec`
/// filesystem object via PyO3.  Lives in the binding layer so that the
/// pure-Rust core never touches Python.
pub struct PyFsspecFileSystem {
    fs: Py<PyAny>,
}

// Safety: Py<PyAny> is Send+Sync in PyO3 ≥ 0.21.  All methods acquire
// the GIL before touching the Python object.
unsafe impl Send for PyFsspecFileSystem {}
unsafe impl Sync for PyFsspecFileSystem {}

impl PyFsspecFileSystem {
    /// Create a filesystem for the given URL via `fsspec.url_to_fs(url)`.
    /// Returns `(PyFsspecFileSystem, stripped_path)`.
    pub fn from_url(url: &str) -> PyResult<(Self, String)> {
        Python::attach(|py| {
            let fsspec = py.import("fsspec")?;
            let result = fsspec.call_method1("url_to_fs", (url,))?;
            let tuple = result.cast::<pyo3::types::PyTuple>()?;
            let py_fs = tuple.get_item(0)?;
            let path = tuple.get_item(1)?.extract::<String>()?;

            Ok((
                Self {
                    fs: py_fs.unbind(),
                },
                path,
            ))
        })
    }
}

// =========================================================================
// FileSystem implementation
// =========================================================================

impl FileSystem for PyFsspecFileSystem {
    fn protocol(&self) -> &[&str] {
        // We can't return references to owned Strings directly as &[&str].
        // Use a leaked static slice (one per instance); acceptable since
        // these are long-lived singletons.  This is a pragmatic trade-off.
        // A more correct approach would be to store &'static str,
        // but since protocols come from Python at run-time we'd need to
        // leak anyway.
        //
        // Instead, return a fixed placeholder.  The real protocol is
        // used in strip_protocol / unstrip_protocol via the Python object.
        &["fsspec"]
    }

    fn strip_protocol(&self, path: &str) -> String {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            // Use the classmethod _strip_protocol
            let cls = fs.getattr("__class__").unwrap();
            cls.call_method1("_strip_protocol", (path,))
                .and_then(|r| r.extract::<String>())
                .unwrap_or_else(|_| path.to_string())
        })
    }

    fn unstrip_protocol(&self, path: &str) -> String {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            fs.call_method1("unstrip_protocol", (path,))
                .and_then(|r| r.extract::<String>())
                .unwrap_or_else(|_| path.to_string())
        })
    }

    fn ls(&self, path: &str, detail: bool) -> FsResult<Vec<FileInfo>> {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            let kwargs = PyDict::new(py);
            kwargs.set_item("detail", detail).map_err(py_err_to_fs)?;
            let result = fs
                .call_method("ls", (path,), Some(&kwargs))
                .map_err(py_err_to_fs)?;

            if detail {
                // Returns list of dicts
                let items: Vec<Bound<PyAny>> = result.extract().map_err(py_err_to_fs)?;
                let mut entries = Vec::with_capacity(items.len());
                for item in &items {
                    let name: String = item
                        .get_item("name")
                        .map_err(py_err_to_fs)?
                        .extract()
                        .map_err(py_err_to_fs)?;
                    let size: u64 = item
                        .get_item("size")
                        .and_then(|v| v.extract())
                        .unwrap_or(0);
                    let type_str: String = item
                        .get_item("type")
                        .map_err(py_err_to_fs)?
                        .extract()
                        .map_err(py_err_to_fs)?;
                    let file_type = match type_str.as_str() {
                        "directory" => FileType::Directory,
                        "file" => FileType::File,
                        _ => FileType::Other,
                    };
                    entries.push(FileInfo {
                        name,
                        size,
                        file_type,
                        created: None,
                        modified: None,
                        extra: HashMap::new(),
                    });
                }
                Ok(entries)
            } else {
                // Returns list of strings — call again with detail=True for full info
                // Actually, fsspec ls with detail=False returns strings.
                // We need FileInfo, so call with detail=True always.
                // But let's handle the string case gracefully.
                let names: Result<Vec<String>, _> = result.extract();
                match names {
                    Ok(names) => {
                        let mut entries = Vec::with_capacity(names.len());
                        for name in names {
                            entries.push(FileInfo {
                                name,
                                size: 0,
                                file_type: FileType::Other,
                                created: None,
                                modified: None,
                                extra: HashMap::new(),
                            });
                        }
                        Ok(entries)
                    }
                    Err(e) => Err(py_err_to_fs(e)),
                }
            }
        })
    }

    fn info(&self, path: &str) -> FsResult<FileInfo> {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            let result = fs.call_method1("info", (path,)).map_err(py_err_to_fs)?;
            let name: String = result
                .get_item("name")
                .map_err(py_err_to_fs)?
                .extract()
                .map_err(py_err_to_fs)?;
            let size: u64 = result
                .get_item("size")
                .and_then(|v| v.extract())
                .unwrap_or(0);
            let type_str: String = result
                .get_item("type")
                .map_err(py_err_to_fs)?
                .extract()
                .map_err(py_err_to_fs)?;
            let file_type = match type_str.as_str() {
                "directory" => FileType::Directory,
                "file" => FileType::File,
                _ => FileType::Other,
            };
            Ok(FileInfo {
                name,
                size,
                file_type,
                created: None,
                modified: None,
                extra: HashMap::new(),
            })
        })
    }

    fn rm_file(&self, path: &str) -> FsResult<()> {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            fs.call_method1("rm_file", (path,))
                .map_err(py_err_to_fs)?;
            Ok(())
        })
    }

    fn cp_file(&self, src: &str, dst: &str) -> FsResult<()> {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            fs.call_method1("cp_file", (src, dst))
                .map_err(py_err_to_fs)?;
            Ok(())
        })
    }

    fn mkdir(&self, path: &str, create_parents: bool) -> FsResult<()> {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            let kwargs = PyDict::new(py);
            kwargs
                .set_item("create_parents", create_parents)
                .map_err(py_err_to_fs)?;
            fs.call_method("mkdir", (path,), Some(&kwargs))
                .map_err(py_err_to_fs)?;
            Ok(())
        })
    }

    fn rmdir(&self, path: &str) -> FsResult<()> {
        Python::attach(|py| {
            let fs = self.fs.bind(py);
            fs.call_method1("rmdir", (path,)).map_err(py_err_to_fs)?;
            Ok(())
        })
    }

    fn open(
        &self,
        _path: &str,
        _mode: OpenMode,
        _options: Option<OpenOptions>,
    ) -> FsResult<Box<dyn FsFile>> {
        Err(FsError::NotSupported(
            "open() not supported via Python fsspec adapter".to_string(),
        ))
    }
}
