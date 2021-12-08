/**
 * \file flow-message/cross/python/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod cow_list;
mod cow_map;
mod iterator;
mod list;
mod map;

use super::*;
use paste::paste;
use pyo3::prelude::*;

const UAM: &str = "Use After Move";

macro_rules! declare_py {
    ($name:ident, $name_str:literal) => {
        paste! {
            #[pyclass(name = $name_str)]
            struct [<Py $name>] {
                inner: Option<$name>
            }
            impl [<Py $name>] {
                fn take(&mut self) -> $name {
                    self.inner.take().expect(UAM)
                }
                fn as_ref(&self) -> &$name {
                    self.inner.as_ref().expect(UAM)
                }
                fn as_mut(&mut self) -> &mut $name {
                    self.inner.as_mut().expect(UAM)
                }
            }
        }
    };
}

declare_py!(List, "List");
declare_py!(Map, "Map");
declare_py!(CowList, "CowList");
declare_py!(CowMap, "CowMap");

#[inline]
fn cvt_idx(len: usize, idx: isize) -> usize {
    if idx < 0 {
        len - idx.abs() as usize
    } else {
        idx as usize
    }
}

fn to_data(py: Python, object: PyObject) -> PyResult<data::Data> {
    let value = object.as_ref(py);
    if value.is_instance::<pyo3::types::PyInt>()? || value.is_instance::<pyo3::types::PyLong>()? {
        let value: i64 = value.extract()?;
        Ok(data::Data::from(value))
    } else if value.is_instance::<pyo3::types::PyBool>()? {
        let value: bool = value.extract()?;
        Ok(data::Data::from(value))
    } else if value.is_instance::<pyo3::types::PyFloat>()? {
        let value: f64 = value.extract()?;
        Ok(data::Data::from(value))
    } else if value.is_instance::<pyo3::types::PyBytes>()?
        || value.is_instance::<pyo3::types::PyByteArray>()?
    {
        let value: Vec<u8> = value.extract()?;
        Ok(data::Data::from(value))
    } else if value.is_instance::<pyo3::types::PyString>()? {
        let value: String = value.extract()?;
        Ok(data::Data::from(value.into_bytes()))
    } else if value.is_instance::<PyList>()? {
        let value: Py<PyList> = value.extract()?;
        let mut b = value.borrow_mut(py);
        Ok(b.take())
    } else if value.is_instance::<PyMap>()? {
        let value: Py<PyMap> = value.extract()?;
        let mut b = value.borrow_mut(py);
        Ok(data::Data::from(b.take()))
    } else if value.is_instance::<PyCowList>()? {
        let value: Py<PyCowList> = value.extract()?;
        let mut b = value.borrow_mut(py);
        Ok(data::Data::from(b.take()))
    } else if value.is_instance::<PyCowMap>()? {
        let value: Py<PyCowMap> = value.extract()?;
        let mut b = value.borrow_mut(py);
        Ok(data::Data::from(b.take()))
    } else {
        unimplemented!()
    }
}

fn from_data(py: Python, data: impl std::ops::Deref<Target = data::Data>) -> PyResult<PyObject> {
    match data.ty() {
        data::DataType::Value(data::ValueType::Int) => {
            let value: i64 = data::Get::get(&*data);
            Ok(value.to_object(py))
        }
        data::DataType::Value(data::ValueType::Float) => {
            let value: f64 = data::Get::get(&*data);
            Ok(value.to_object(py))
        }
        data::DataType::Value(data::ValueType::Bool) => {
            let value: bool = data::Get::get(&*data);
            Ok(value.to_object(py))
        }
        data::DataType::Value(data::ValueType::Byte) => {
            let value: u8 = data::Get::get(&*data);
            Ok(value.to_object(py))
        }
        data::DataType::Collection(data::CollectionType::List(_)) => {
            let list = data.clone();
            Py::new(py, PyList { inner: Some(list) }).map(|inner| inner.to_object(py))
        }
        data::DataType::Collection(data::CollectionType::Map) => {
            let map: &Map = data::GetRef::get_ref(&*data);
            Py::new(
                py,
                PyMap {
                    inner: Some(map.to_owned()),
                },
            )
            .map(|inner| inner.to_object(py))
        }
        data::DataType::Collection(data::CollectionType::CowList) => {
            let map: &CowList = data::GetRef::get_ref(&*data);
            Py::new(
                py,
                PyCowList {
                    inner: Some(map.to_owned()),
                },
            )
            .map(|inner| inner.to_object(py))
        }
        data::DataType::Collection(data::CollectionType::CowMap) => {
            let map: &CowMap = data::GetRef::get_ref(&*data);
            Py::new(
                py,
                PyCowMap {
                    inner: Some(map.to_owned()),
                },
            )
            .map(|inner| inner.to_object(py))
        }
        _ => unimplemented!(),
    }
}
