/**
 * \file flow-rs/src/loader/python/channel.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::context::with_context;
use super::envelope::PyEnvelope;
use crate::channel::{Receiver, Sender};
use pyo3::prelude::*;
use stackful::wait;
use std::{sync::Arc};

#[pyclass]
pub(crate) struct PySender {
    pub imp: Arc<Sender>,
}

#[pyclass]
pub(crate) struct PyReceiver {
    pub imp: Arc<Receiver>,
}

#[pymethods]
impl PySender {
    fn send(&mut self, py: Python, envelope: Py<PyEnvelope>) {
        let envelope = envelope.borrow_mut(py).imp.take().expect("use after move");
        with_context(py, || wait(self.imp.send(envelope)).ok());
    }
}

#[pymethods]
impl PyReceiver {
    fn recv(&mut self, py: Python) -> PyObject {
        match with_context(py, || wait(self.imp.recv::<PyObject>())) {
            Ok(msg) => Py::new(py, PyEnvelope { imp: Some(msg) })
                .unwrap()
                .to_object(py),
            _ => py.None(),
        }
    }
}
