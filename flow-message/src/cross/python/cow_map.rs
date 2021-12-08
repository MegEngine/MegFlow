/**
 * \file flow-message/cross/python/cow_map.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::*;
use pyo3::{PyIterProtocol, PyMappingProtocol};

#[pyproto]
impl PyMappingProtocol for PyCowMap {
    fn __len__(&self) -> usize {
        self.as_ref().len()
    }
    fn __setitem__(&mut self, attr: String, value: PyObject) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let data = to_data(py, value)?;
        self.as_mut().insert(attr, data);
        Ok(())
    }

    fn __getitem__(&self, attr: &str) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Some(data) = self.as_ref().get(attr) {
            from_data(py, data)
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err("Key Error"))
        }
    }

    fn __delitem__(&mut self, attr: &str) -> PyResult<()> {
        self.as_mut().remove(attr);
        Ok(())
    }
}

#[pyproto]
impl PyIterProtocol for PyCowMap {
    fn __iter__(this: PyRefMut<Self>) -> PyResult<super::iterator::PyObjectPairIterator> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut elements = std::vec::Vec::new();
        for (k, v) in this.as_ref().iter() {
            elements.push((k.to_object(py), from_data(py, &v.clone())?));
        }

        Ok(super::iterator::PyObjectPairIterator::new(
            elements.into_iter(),
        ))
    }
}
