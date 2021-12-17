/**
 * \file flow-message/cross/python/iterator.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
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
