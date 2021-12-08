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
use flow_rs::loader;
use flow_rs::prelude::*;
use pyo3::prelude::*;
use std::env::current_dir;
use std::path::PathBuf;

#[pyo3::pyclass]
#[derive(Default)]
pub struct RunConfig {
    #[pyo3(get, set)]
    debug: Option<String>,
    #[pyo3(get, set)]
    dump: bool,
    #[pyo3(get, set)]
    module_path: Option<PathBuf>,
    #[pyo3(get, set)]
    plugin_path: Option<PathBuf>,
    #[pyo3(get, set)]
    config_path: Option<PathBuf>,
}

#[pymethods]
impl RunConfig {
    #[new]
    fn new() -> Self {
        Default::default()
    }
}

impl RunConfig {
    fn take(&mut self) -> Self {
        Self {
            debug: self.debug.take(),
            dump: self.dump,
            module_path: self.module_path.take(),
            plugin_path: self.plugin_path.take(),
            config_path: self.config_path.take(),
        }
    }
}

async fn run_impl(run_cfg: RunConfig) {
    let module = run_cfg
        .module_path
        .unwrap_or_else(|| current_dir().unwrap());
    let plugin = run_cfg.plugin_path.unwrap();
    let config = run_cfg.config_path.unwrap_or_else(|| {
        let mut config = std::fs::canonicalize(&plugin).unwrap();
        let dirname = config.file_name().unwrap();
        let config_name = format!("{}.toml", dirname.to_str().unwrap());
        config.push(config_name);
        config
    });

    flow_plugins::export();
    ctrlc::set_handler(|| unsafe { libc::_exit(0) }).expect("Error setting Ctrl-C handler");

    if run_cfg.dump {
        let mut dump_path = config.clone();
        dump_path.pop();
        let file_stem = config.file_stem().unwrap().to_str().unwrap();
        dump_path.push(format!("{}.png", file_stem));
        log::info!("dump path: {:?}", dump_path);
        std::env::set_var("MEGFLOW_DUMP", dump_path.to_str().unwrap());
    }

    let plugin_cfg = loader::LoaderConfig {
        module_path: module,
        plugin_path: plugin,
        ty: loader::PluginType::Python,
    };
    #[allow(unused_variables)]
    if let Some(dbg_port) = run_cfg.debug {
        #[cfg(feature = "debug")]
        {
            let dbg_port: u16 = dbg_port.parse().unwrap();
            let server = flow_rs::DebugServer::open(&config, dbg_port).unwrap();
            flow_rs::rt::task::spawn(async move {
                server.listen().await;
            });
        }
        #[cfg(not(feature = "debug"))]
        {
            unimplemented!("not supported (build option without feature[debug])")
        }
    }

    let mut graph = load(Some(plugin_cfg), &config).unwrap();
    let graph_handle = graph.start();
    graph.stop();
    graph_handle.await
}

#[pyo3::pyfunction]
pub fn run(py: Python, py_run_cfg: Py<RunConfig>) {
    let run_cfg = py_run_cfg.borrow_mut(py).take();
    py.allow_threads(|| {
        flow_rs::rt::task::block_on(async move {
            run_impl(run_cfg).await;
        });
    })
}

#[pymodule]
fn megflow(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RunConfig>()?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(())
}
