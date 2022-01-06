/**
 * \file flow-rs/src/loader/python/port.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::channel::Owned2PyObject;
use crate::config::interlayer::PortTy;
use pyo3::prelude::*;
use std::collections::BTreeMap;

pub(super) struct AnyPort<T> {
    pub storage: BTreeMap<u64, T>,
    pub ty: PortTy,
    pub name: String,
}

impl<T> AnyPort<T> {
    fn new(name: String, ty: PortTy) -> Self {
        AnyPort {
            storage: Default::default(),
            ty,
            name,
        }
    }
}

impl<T> AnyPort<T>
where
    T: Owned2PyObject + Clone + Default,
{
    fn to_list(&self, py: Python) -> PyResult<PyObject> {
        use pyo3::types::PyList;
        let objects: PyResult<Vec<_>> = self
            .storage
            .values()
            .cloned()
            .map(|x| x.into_object(py))
            .collect();
        Ok(PyList::new(py, objects?).to_object(py))
    }

    fn to_object(&self, py: Python) -> PyResult<PyObject> {
        self.storage
            .values()
            .next()
            .cloned()
            .map(|x| x.into_object(py))
            .unwrap_or_else(|| T::default().into_object(py))
    }

    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        use pyo3::types::IntoPyDict;
        let mut list = vec![];
        for (k, v) in self.storage.iter() {
            let v = v.clone().into_object(py)?;
            let k = k.to_object(py);
            list.push((k, v));
        }
        Ok(list.into_py_dict(py).to_object(py))
    }
}

impl<T> ToPyObject for AnyPort<T>
where
    T: Owned2PyObject + Clone + Default,
{
    fn to_object(&self, py: Python) -> PyObject {
        match self.ty {
            PortTy::Dyn => unimplemented!(),
            PortTy::Unit => self.to_object(py).expect("to python normal port fault"),
            PortTy::List => self.to_list(py).expect("to python list port fault"),
            PortTy::Dict => self.to_dict(py).expect("to python dict port fault"),
        }
    }
}

pub(super) fn parse<T>(s: &str) -> (String, AnyPort<std::sync::Arc<T>>) {
    let mut ss = s.split(':');
    let name = ss.next().unwrap();
    ss.next()
        .map(|x| {
            if x == "[]" {
                (
                    format!("[{}]", name),
                    AnyPort::new(name.to_owned(), PortTy::List),
                )
            } else if x == "{}" {
                (
                    format!("{{{}}}", name),
                    AnyPort::new(name.to_owned(), PortTy::Dict),
                )
            } else {
                unreachable!("unexpected port type {}", x)
            }
        })
        .unwrap_or_else(|| (name.to_owned(), AnyPort::new(name.to_owned(), PortTy::Unit)))
}

pub(super) fn port_name(s: &str) -> String {
    let mut ss = s.split(':');
    let name = ss.next().unwrap();
    ss.next()
        .map(|x| {
            if x == "[]" {
                format!("[{}]", name)
            } else if x == "{}" {
                format!("{{{}}}", name)
            } else {
                unreachable!("unexpected port type {}", x)
            }
        })
        .unwrap_or_else(|| name.to_owned())
}
