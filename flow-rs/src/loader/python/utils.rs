/**
 * \file flow-rs/src/loader/python/utils.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::context::with_context;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use stackful::*;
use std::time::Duration;

#[pyfunction]
fn block_on(py: Python, task: PyObject) {
    py.allow_threads(move || {
        flow_rs::rt::task::block_on(flow_rs::rt::task::spawn_local(stackful(move || {
            Python::with_gil(move |py| task.call0(py).unwrap())
        })));
    })
}

#[pyfunction]
fn yield_now(py: Python) {
    with_context(py, || wait(crate::rt::task::yield_now()))
}

#[pyfunction]
fn sleep(py: Python, dur: u64) {
    with_context(py, || {
        wait(crate::rt::task::sleep(Duration::from_millis(dur)))
    })
}

#[pyfunction]
fn join(py: Python, tasks: Vec<PyObject>) -> Vec<PyObject> {
    let mut futs = vec![];
    for task in tasks.into_iter() {
        futs.push(flow_rs::rt::task::spawn_local(stackful(move || {
            Python::with_gil(|py| task.call0(py).unwrap())
        })));
    }
    with_context(py, || wait(futures_util::future::join_all(futs)))
}

#[pyclass(name = "Future", unsendable)]
struct PyFuture {
    chan: Option<oneshot::Receiver<PyObject>>,
}

#[pyclass(name = "Waker")]
struct PyWaker {
    chan: Option<oneshot::Sender<PyObject>>,
}

#[pymethods]
impl PyFuture {
    fn wait(&mut self, py: Python) -> PyObject {
        if let Some(chan) = std::mem::take(&mut self.chan) {
            with_context(py, || wait(chan)).unwrap()
        } else {
            py.None()
        }
    }

    fn cancel(&mut self) {
        self.chan = None;
    }
}

#[pymethods]
impl PyWaker {
    fn wake(&mut self, py: Python, result: PyObject) {
        if let Some(chan) = std::mem::take(&mut self.chan) {
            let _ = chan.send(result.clone_ref(py)).is_ok();
        }
    }
}

#[pyfunction]
fn create_future() -> (PyFuture, PyWaker) {
    let (s, r) = oneshot::channel();
    (PyFuture { chan: Some(r) }, PyWaker { chan: Some(s) })
}

pub fn utils_register(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(yield_now, module)?)?;
    module.add_function(wrap_pyfunction!(sleep, module)?)?;
    module.add_function(wrap_pyfunction!(join, module)?)?;
    module.add_function(wrap_pyfunction!(create_future, module)?)?;
    module.add_function(wrap_pyfunction!(block_on, module)?)?;
    module.add_class::<PyFuture>()?;
    module.add_class::<PyWaker>()?;

    Ok(())
}
