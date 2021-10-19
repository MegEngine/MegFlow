use pyo3::prelude::{pyclass, pyproto};
use pyo3::{Py, PyIterProtocol, PyObject, PyRefMut, PyResult};

#[pyclass]
pub struct PyObjectIterator {
    iterator: std::vec::IntoIter<PyObject>,
}

impl PyObjectIterator {
    #[must_use]
    pub fn new(iterator: std::vec::IntoIter<PyObject>) -> Self {
        PyObjectIterator { iterator }
    }
}

#[pyproto]
impl PyIterProtocol for PyObjectIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<PyObjectIterator>> {
        Ok(slf.into())
    }
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        Ok(slf.iterator.next())
    }
}

#[pyclass]
pub struct PyObjectPairIterator {
    iterator: std::vec::IntoIter<(PyObject, PyObject)>,
}

impl PyObjectPairIterator {
    #[must_use]
    pub fn new(iterator: std::vec::IntoIter<(PyObject, PyObject)>) -> Self {
        PyObjectPairIterator { iterator }
    }
}

#[pyproto]
impl PyIterProtocol for PyObjectPairIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<PyObjectPairIterator>> {
        Ok(slf.into())
    }
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<(PyObject, PyObject)>> {
        Ok(slf.iterator.next())
    }
}
