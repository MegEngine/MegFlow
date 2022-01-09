/**
 * \file flow-python/src/lib.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use anyhow::Result;
use flow_rs::loader::python::channel::*;
use flow_rs::loader::python::envelope::envelope_register;
use flow_rs::loader::python::utils::utils_register;
use flow_rs::prelude::*;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Once};

#[pyclass(module = "megflow")]
pub struct Graph {
    graph: Option<MainGraph>,
    handle: Option<flow_rs::rt::task::JoinHandle<Result<()>>>,
    inps: HashMap<String, PyObject>,
    outs: HashMap<String, PyObject>,
}

static ONCE_INIT: Once = Once::new();

#[pymethods]
impl Graph {
    fn wait(&mut self, py: Python) {
        self.inps.clear();
        self.outs.clear();
        if let Some(graph) = self.graph.take() {
            graph.stop();
        }
        if let Some(handle) = self.handle.take() {
            py.allow_threads(|| {
                flow_rs::rt::task::block_on(async {
                    handle.await.unwrap();
                });
            });
        }
    }

    fn inputs(&self) -> Vec<&str> {
        self.inps.keys().map(|x| x.as_str()).collect()
    }

    fn outputs(&self) -> Vec<&str> {
        self.outs.keys().map(|x| x.as_str()).collect()
    }

    fn input(&self, key: &str) -> Option<PyObject> {
        self.inps.get(key).cloned()
    }

    fn output(&self, key: &str) -> Option<PyObject> {
        self.outs.get(key).cloned()
    }

    fn close(&mut self) {
        self.inps.clear();
        if let Some(graph) = self.graph.take() {
            graph.stop();
        }
    }

    #[new]
    #[args(
        config_path = "None",
        config_str = "None",
        dynamic_path = "None",
        dynamic_str = "None",
        plugin_path = "None",
        module_path = "None",
        dump = "false"
    )]
    fn new(
        mut config_path: Option<PathBuf>,
        config_str: Option<&str>,
        dynamic_path: Option<PathBuf>,
        dynamic_str: Option<&str>,
        plugin_path: Option<PathBuf>,
        module_path: Option<PathBuf>,
        dump: bool,
    ) -> PyResult<Graph> {
        ONCE_INIT.call_once(|| {
            // workaround for https://github.com/rust-lang/rust/issues/47384
            flow_plugins::export();
            ctrlc::set_handler(|| unsafe { libc::_exit(0) }).expect("Error setting Ctrl-C handler");
        });

        // load graph
        if let Some(plugin_path) = &plugin_path {
            if config_str.is_none() && config_path.is_none() {
                let mut config = std::fs::canonicalize(plugin_path).unwrap();
                let dirname = config.file_name().unwrap();
                let config_name = format!("{}.toml", dirname.to_str().unwrap());
                config.push(config_name);
                config_path = Some(config);
            }
        }

        if dump {
            if let Some(config) = &config_path {
                let mut dump_path = config.clone();
                dump_path.pop();
                let file_stem = config.file_stem().unwrap().to_str().unwrap();
                dump_path.push(format!("{}.png", file_stem));
                log::info!("dump path: {:?}", dump_path);
                std::env::set_var("MEGFLOW_DUMP", dump_path.to_str().unwrap());
            } else {
                let mut dump_path = std::env::current_dir().unwrap();
                dump_path.push("graph.png");
                log::info!("dump path: {:?}", dump_path);
                std::env::set_var("MEGFLOW_DUMP", dump_path.to_str().unwrap());
            }
        }

        let plugin_cfg = flow_rs::loader::LoaderConfig {
            module_path,
            plugin_path,
            ty: flow_rs::loader::PluginType::Python,
        };

        let mut builder = Builder::new();
        if let Some(config_path) = config_path {
            builder = builder
                .template_file(&config_path)
                .unwrap_or_else(|_| panic!("file[{:?}] not found", config_path));
        }
        if let Some(config_str) = config_str {
            builder = builder.template(config_str.to_owned());
        }
        if let Some(dynamic_path) = dynamic_path {
            builder = builder
                .dynamic_file(&dynamic_path)
                .unwrap_or_else(|_| panic!("file[{:?}] not found", dynamic_path));
        }
        if let Some(dynamic_str) = dynamic_str {
            builder = builder.dynamic(dynamic_str.to_owned());
        }

        let mut graph = builder.load_plugins(plugin_cfg).build().unwrap();

        // prepare io
        let mut inps = HashMap::new();
        let mut outs = HashMap::new();

        Python::with_gil(|py| -> PyResult<()> {
            for name in graph.input_names() {
                let inp = graph.input(name).unwrap();
                inps.insert(name.to_owned(), Arc::new(inp).into_object(py)?);
            }

            for name in graph.output_names() {
                let out = graph.output(name).unwrap();
                outs.insert(name.to_owned(), Arc::new(out).into_object(py)?);
            }
            Ok(())
        })
        .unwrap();

        // run graph
        let handle = graph.start();

        Ok(Graph {
            graph: Some(graph),
            handle: Some(handle),
            inps,
            outs,
        })
    }
}

#[pymodule]
fn megflow(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Graph>()?;
    utils_register(m)?;
    envelope_register(m)?;
    Ok(())
}
