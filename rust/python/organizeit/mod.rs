use pyo3::prelude::*;

use ::organizeit2::OrganizeIt as BaseOrganizeIt;

use super::directory::Directory;

#[pyclass]
pub struct OrganizeIt {
    _inner: BaseOrganizeIt,
}

#[pymethods]
impl OrganizeIt {
    #[new]
    fn new() -> Self {
        OrganizeIt {
            _inner: BaseOrganizeIt::new(),
        }
    }

    fn expand(&self, directory: String) -> Directory {
        Directory {
            inner: self._inner.expand(&directory),
        }
    }
}
