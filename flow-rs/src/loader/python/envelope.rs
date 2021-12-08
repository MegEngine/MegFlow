/**
 * \file flow-rs/src/loader/python/envelope.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::{
    envelope::{AnyEnvelope, Envelope},
    prelude::EnvelopeInfo,
};
use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyDict},
};
use std::sync::Arc;

static ERR_MSG: &str = "use after move";

#[pyclass(name = "Envelope", module = "megflow")]
pub(crate) struct PyEnvelope {
    pub imp: Option<Envelope<PyObject>>,
}

#[pymethods]
impl PyEnvelope {
    #[new]
    fn new() -> Self {
        PyEnvelope { imp: None }
    }

    #[staticmethod]
    #[args(info = "None")]
    fn pack(py: Python, msg: PyObject, info: Option<&PyDict>) -> PyResult<Self> {
        let mut envelope = if msg.is_none(py) {
            Envelope::<PyObject>::empty()
        } else {
            Envelope::new(msg)
        };

        if let Some(info) = info {
            let target = envelope.info_mut();
            macro_rules! restore {
                ($k:ident) => {
                    if let Some($k) = info.get_item(stringify!($k)) {
                        target.$k = $k.extract()?;
                    }
                };
            }
            restore!(from_addr);
            restore!(to_addr);
            restore!(transfer_addr);
            restore!(partial_id);
            restore!(tag);
            if let Some(extra_data) = info.get_item("extra_data") {
                target.extra_data = Some(Arc::new(extra_data.to_object(py)))
            }
        }
        Ok(PyEnvelope {
            imp: Some(envelope),
        })
    }

    fn repack(&self, msg: PyObject) -> Self {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        PyEnvelope {
            imp: Some(envelope.repack(msg)),
        }
    }

    #[getter(extra_data)]
    fn get_extra_data(&self) -> Option<PyObject> {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        envelope
            .info()
            .extra_data
            .as_ref()
            .map(|x| x.downcast_ref().cloned())
            .flatten()
    }

    #[getter(msg)]
    fn get_msg(&self, py: Python) -> PyObject {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        envelope.get_ref().clone_ref(py)
    }

    #[setter(msg)]
    fn set_msg(&mut self, msg: PyObject) {
        let envelope = self.imp.as_mut().expect(ERR_MSG);
        envelope.repack_inplace(msg)
    }

    #[getter(from_addr)]
    fn get_from_addr(&self) -> Option<u64> {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        envelope.info().from_addr
    }

    #[setter(from_addr)]
    fn set_from_addr(&mut self, addr: u64) {
        let envelope = self.imp.as_mut().expect(ERR_MSG);
        envelope.info_mut().from_addr = Some(addr)
    }

    #[getter(to_addr)]
    fn get_to_addr(&self) -> Option<u64> {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        envelope.info().to_addr
    }

    #[setter(to_addr)]
    fn set_to_addr(&mut self, addr: u64) {
        let envelope = self.imp.as_mut().expect(ERR_MSG);
        envelope.info_mut().to_addr = Some(addr)
    }

    #[getter(partial_id)]
    fn get_partial_id(&self) -> Option<u64> {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        envelope.info().partial_id
    }

    #[getter(tag)]
    fn get_tag(&self) -> Option<&String> {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        envelope.info().tag.as_ref()
    }

    #[setter(tag)]
    fn set_tag(&mut self, tag: String) {
        let envelope = self.imp.as_mut().expect(ERR_MSG);
        envelope.info_mut().tag = Some(tag);
    }
}

impl IntoPyDict for EnvelopeInfo {
    fn into_py_dict(self, py: Python) -> &PyDict {
        let dict = PyDict::new(py);
        macro_rules! store {
            ($k: ident) => {
                dict.set_item(stringify!($k), self.$k).unwrap();
            };
        }
        store!(from_addr);
        store!(to_addr);
        store!(transfer_addr);
        store!(partial_id);
        store!(tag);
        dict
    }
}

pub fn envelope_register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyEnvelope>()?;
    Ok(())
}
