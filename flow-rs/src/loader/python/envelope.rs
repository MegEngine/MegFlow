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

static ERR_MSG: &str = "use after move";

#[pyclass(name = "Envelope", module = "pyflow")]
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
            macro_rules! restore {
                ($k:ident) => {
                    envelope.info_mut().$k = info
                        .get_item(stringify!($k))
                        .expect(concat!("expect ", stringify!($k), " field"))
                        .extract()?;
                };
            }
            restore!(from_addr);
            restore!(to_addr);
            restore!(transfer_addr);
            restore!(partial_id);
        }
        Ok(PyEnvelope {
            imp: Some(envelope),
        })
    }

    fn __getstate__(&mut self, py: Python) -> PyResult<PyObject> {
        let envelope = self.imp.as_mut().expect(ERR_MSG);
        let dict = envelope.info().into_py_dict(py);
        dict.set_item("msg", envelope.get_mut().clone_ref(py))?;
        Ok(dict.to_object(py))
    }

    fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        let dict: &PyDict = state.extract(py)?;
        let msg: PyObject = dict.get_item("msg").expect("expect msg field").into();
        let envelope = PyEnvelope::pack(py, msg, Some(dict))?;
        self.imp = envelope.imp;
        Ok(())
    }

    fn repack(&self, msg: PyObject) -> Self {
        let envelope = self.imp.as_ref().expect(ERR_MSG);
        PyEnvelope {
            imp: Some(envelope.repack(msg)),
        }
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

    #[setter(partial_id)]
    fn set_partial_id(&mut self, addr: u64) {
        let envelope = self.imp.as_mut().expect(ERR_MSG);
        envelope.info_mut().partial_id = Some(addr)
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
        dict
    }
}

pub fn envelope_register(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyEnvelope>()?;
    Ok(())
}
