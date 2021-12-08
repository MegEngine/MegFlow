/**
 * \file flow-message/cross/python/list.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::*;
use pyo3::PySequenceProtocol;

#[pyproto]
impl PySequenceProtocol for PyList {
    fn __len__(&self) -> usize {
        self.as_ref().len()
    }

    fn __setitem__(&mut self, idx: isize, value: PyObject) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let value = value.as_ref(py);
        let len = self.as_ref().len();
        let idx = cvt_idx(len, idx);
        if value.is_instance::<pyo3::types::PyInt>()?
            || value.is_instance::<pyo3::types::PyLong>()?
        {
            let value: i64 = value.extract()?;
            match self.as_ref().ty() {
                data::DataType::Collection(data::CollectionType::List(data::ValueType::Byte)) => {
                    self.as_mut().xset(idx, value as u8);
                }
                _ => self.as_mut().xset(idx, value),
            }
        } else if value.is_instance::<pyo3::types::PyBool>()? {
            let value: bool = value.extract()?;
            self.as_mut().xset(idx, value);
        } else if value.is_instance::<pyo3::types::PyFloat>()? {
            let value: f64 = value.extract()?;
            self.as_mut().xset(idx, value);
        } else {
            unimplemented!()
        }
        Ok(())
    }

    fn __getitem__(&self, idx: isize) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let len = self.as_ref().len();
        let idx = cvt_idx(len, idx);

        match self.as_ref().ty() {
            data::DataType::Collection(data::CollectionType::List(data::ValueType::Int)) => {
                let value: PyResult<i64> = self
                    .as_ref()
                    .xget(&(idx))
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err("Idx out of bound"));
                value.map(|value| value.to_object(py))
            }
            data::DataType::Collection(data::CollectionType::List(data::ValueType::Float)) => {
                let value: PyResult<f64> = self
                    .as_ref()
                    .xget(&(idx))
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err("Idx out of bound"));
                value.map(|value| value.to_object(py))
            }
            data::DataType::Collection(data::CollectionType::List(data::ValueType::Bool)) => {
                let value: PyResult<bool> = self
                    .as_ref()
                    .xget(&(idx))
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err("Idx out of bound"));
                value.map(|value| value.to_object(py))
            }
            data::DataType::Collection(data::CollectionType::List(data::ValueType::Byte)) => {
                let value: PyResult<u8> = self
                    .as_ref()
                    .xget(&(idx))
                    .ok_or_else(|| pyo3::exceptions::PyTypeError::new_err("Idx out of bound"));
                value.map(|value| value.to_object(py))
            }
            _ => {
                unimplemented!()
            }
        }
    }
}
