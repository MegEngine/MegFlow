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
use crate::channel::{BatchRecvError, Receiver, Sender};
use pyo3::prelude::*;
use stackful::wait;
use std::{sync::Arc, time::Duration};

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

    fn batch_recv(&self, py: Python, n: usize, dur: u64) -> (Vec<PyObject>, bool) {
        let convert = |envelope| {
            Py::new(
                py,
                PyEnvelope {
                    imp: Some(envelope),
                },
            )
            .unwrap()
            .to_object(py)
        };
        match with_context(py, || {
            wait(
                self.imp
                    .batch_recv::<PyObject>(n, Duration::from_millis(dur)),
            )
        }) {
            Ok(msg) => (msg.into_iter().map(convert).collect(), false),
            Err(BatchRecvError::Closed(msg)) => (msg.into_iter().map(convert).collect(), true),
        }
    }
}

pub trait Owned2PyObject {
    fn into_object(self, py: Python) -> PyResult<PyObject>;
}

impl Owned2PyObject for std::sync::Arc<Sender> {
    fn into_object(self, py: Python) -> PyResult<PyObject> {
        let sender = PySender { imp: self };
        Ok(PyCell::new(py, sender)?.to_object(py))
    }
}

impl Owned2PyObject for std::sync::Arc<Receiver> {
    fn into_object(self, py: Python) -> PyResult<PyObject> {
        let receiver = PyReceiver { imp: self };
        Ok(PyCell::new(py, receiver)?.to_object(py))
    }
}
