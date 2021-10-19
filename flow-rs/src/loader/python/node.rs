/**
 * \file flow-rs/src/loader/python/node.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::channel::*;
use super::RegistryNodeParams;
use flow_rs::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use stackful::stackful;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) struct PyNode {
    imp: PyObject,
    inputs: HashMap<String, Arc<Receiver>>,
    outputs: HashMap<String, Arc<Sender>>,
    name: String,
    exclusive: bool,
}

impl Drop for PyNode {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            if !self.imp.as_ref(py).hasattr("__del__").unwrap() {
                return;
            }
            if let Err(err) = self.imp.call_method0(py, "__del__") {
                err.print(py);
                panic!("python node {} __del__ fault!", self.name);
            }
        });
    }
}

impl PyNode {
    pub fn new(
        instance_name: String,
        args: &toml::value::Table,
        params: &RegistryNodeParams,
    ) -> PyNode {
        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();
        for port in &params.inputs {
            inputs.insert(port.clone(), Arc::new(Default::default()));
        }
        for port in &params.outputs {
            outputs.insert(port.clone(), Arc::new(Default::default()));
        }
        let imp = Python::with_gil(|py| -> _ {
            let pyargs = toml2dict(py, args).expect("convert toml to python dict fault");
            match params.code.call1(py, (instance_name.as_str(), pyargs)) {
                Err(err) => {
                    err.print(py);
                    panic!("parse python code fault");
                }
                Ok(ret) => ret,
            }
        });
        PyNode {
            inputs,
            outputs,
            imp,
            name: params.name.clone(),
            exclusive: params.exclusive,
        }
    }

    async fn initialize(&mut self, res: ResourceCollection) {
        let inputs = self.inputs.clone();
        let outputs = self.outputs.clone();

        Python::with_gil(|py| {
            for (name, input) in inputs {
                let rp = PyReceiver { imp: input };
                let cell = PyCell::new(py, rp).unwrap();
                self.imp.as_ref(py).setattr(name, cell).unwrap();
            }
            for (name, output) in outputs {
                let sp = PySender { imp: output };
                let cell = PyCell::new(py, sp).unwrap();
                self.imp.as_ref(py).setattr(name, cell).unwrap();
            }
        });

        for k in res.keys() {
            let any_r = res
                .get_any(k.as_str())
                .await
                .unwrap_or_else(|| panic!("resource {} not found", k));
            Python::with_gil(|py| {
                let r = any_r.to_python(py);
                self.imp.as_ref(py).setattr(k, r).unwrap();
            });
        }
    }

    async fn exec(&mut self) {
        stackful(|| {
            Python::with_gil(|py| {
                if let Err(err) = self.imp.call_method0(py, "exec") {
                    err.print(py);
                    panic!("python node {} exec fault!", self.name);
                }
            })
        })
        .await;
    }

    async fn start_loop(&mut self, res: ResourceCollection) {
        self.initialize(res).await;
        let mut empty_n = 0;
        loop {
            self.exec().await;
            if !self.inputs.is_empty() {
                let mut min_empty_n = usize::MAX;
                for port in self.inputs.values() {
                    min_empty_n = std::cmp::min(min_empty_n, port.empty_n());
                }

                for _ in empty_n..min_empty_n {
                    for port in self.outputs.values() {
                        port.send_any(DummyEnvelope {}.seal()).await.ok();
                    }
                }

                empty_n = min_empty_n;
            }
            if self.is_allinp_closed() {
                break;
            }
        }
        self.close();
    }
}

impl Node for PyNode {
    fn set_port(&mut self, port_name: &str, _: Option<u64>, channel: &ChannelStorage) {
        if let Some(port) = self.inputs.get_mut(port_name) {
            *port = Arc::new(channel.receiver());
        } else if let Some(port) = self.outputs.get_mut(port_name) {
            *port = Arc::new(channel.sender());
        } else {
            unreachable!();
        }
    }
    fn set_port_dynamic(&mut self, _: u64, _: &str, _: String, _: usize, _: Vec<BrokerClient>) {
        unimplemented!()
    }
    fn close(&mut self) {
        for port in self.outputs.values_mut() {
            unsafe { &mut *(Arc::as_ptr(port) as *mut Sender) }.abort();
        }
    }
    fn is_allinp_closed(&self) -> bool {
        let mut is_closed = true;
        for port in self.inputs.values() {
            is_closed = is_closed && port.is_closed();
        }
        is_closed
    }
}

impl Actor for PyNode {
    fn start(mut self: Box<Self>, _: Context, res: ResourceCollection) -> rt::task::JoinHandle<()> {
        if self.exclusive {
            flow_rs::rt::task::spawn_blocking(move || {
                flow_rs::rt::task::block_on(async move {
                    self.start_loop(res).await;
                });
            })
        } else {
            flow_rs::rt::task::spawn_local(async move {
                self.start_loop(res).await;
            })
        }
    }
}

pub fn toml2dict<'a>(py: Python<'a>, args: &toml::value::Table) -> PyResult<&'a PyDict> {
    fn append_list(py: Python, value: &toml::Value, list: &PyList) -> PyResult<()> {
        match value {
            toml::Value::String(s) => list.append(s),
            toml::Value::Float(f) => list.append(f),
            toml::Value::Integer(i) => list.append(i),
            toml::Value::Boolean(b) => list.append(b),
            toml::Value::Datetime(t) => list.append(t.to_string()),
            toml::Value::Array(l) => {
                let pylist = PyList::empty(py);
                for e in l {
                    append_list(py, e, pylist)?;
                }
                list.append(pylist)
            }
            toml::Value::Table(d) => {
                let pydict = PyDict::new(py);
                for (key, value) in d {
                    fill_dict(py, key, value, pydict)?;
                }
                list.append(pydict)
            }
        }
    }
    fn fill_dict(py: Python, key: &str, value: &toml::Value, dict: &PyDict) -> PyResult<()> {
        match value {
            toml::Value::String(s) => dict.set_item(key, s),
            toml::Value::Float(f) => dict.set_item(key, f),
            toml::Value::Integer(i) => dict.set_item(key, i),
            toml::Value::Boolean(b) => dict.set_item(key, b),
            toml::Value::Datetime(t) => dict.set_item(key, t.to_string()),
            toml::Value::Array(l) => {
                let pylist = PyList::empty(py);
                for e in l {
                    append_list(py, e, pylist)?;
                }
                dict.set_item(key, pylist)
            }
            toml::Value::Table(d) => {
                let pydict = PyDict::new(py);
                for (key, value) in d {
                    fill_dict(py, key, value, pydict)?;
                }
                dict.set_item(key, pydict)
            }
        }
    }
    let dict = PyDict::new(py);
    for (key, value) in args {
        fill_dict(py, key, value, dict)?;
    }
    Ok(dict)
}
