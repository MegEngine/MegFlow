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
use pyo3::types::PyFunction;
use pyo3::wrap_pyfunction;
use stackful::wait;
use std::time::Duration;

#[pyclass(name = "Future")]
struct PyFuture {
    chan: Option<oneshot::Receiver<PyObject>>,
}

#[pyclass(name = "Waker")]
struct PyWaker {
    chan: Option<oneshot::Sender<PyObject>>,
    callback: Option<Py<PyFunction>>,
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
    fn wake(&mut self, py: Python, result: PyObject) -> PyResult<()> {
        if let Some(chan) = std::mem::take(&mut self.chan) {
            if chan.send(result.clone_ref(py)).is_ok() {
                if let Some(callback) = std::mem::take(&mut self.callback) {
                    callback.call1(py, (result,))?;
                }
            }
        }
        Ok(())
    }
}

#[pyfunction(callback = "None")]
fn create_future(callback: Option<Py<PyFunction>>) -> (PyFuture, PyWaker) {
    let (s, r) = oneshot::channel();
    (
        PyFuture { chan: Some(r) },
        PyWaker {
            chan: Some(s),
            callback,
        },
    )
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
fn join(py: Python, tasks: Vec<Py<PyFunction>>) -> Vec<PyObject> {
    let mut futs = vec![];
    for task in tasks {
        futs.push(async move { Python::with_gil(|py| task.call0(py).unwrap()) });
    }
    with_context(py, || wait(futures_util::future::join_all(futs)))
}

pub fn utils_register(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(yield_now, module)?)?;
    module.add_function(wrap_pyfunction!(sleep, module)?)?;
    module.add_function(wrap_pyfunction!(join, module)?)?;
    module.add_function(wrap_pyfunction!(create_future, module)?)?;
    module.add_class::<PyFuture>()?;
    module.add_class::<PyWaker>()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[flow_rs::rt::test]
    async fn test_future() -> PyResult<()> {
        pyo3::prepare_freethreaded_python();
        let (mut fut, mut waker) = create_future(None);
        Python::with_gil(|py| -> PyResult<_> {
            waker.wake(py, 1usize.into_py(py))?;
            let ret: usize = fut.wait(py).extract(py)?;
            assert_eq!(ret, 1);
            Ok(())
        })?;
        Ok(())
    }
}
