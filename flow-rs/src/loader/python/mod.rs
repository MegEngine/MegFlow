/**
 * \file flow-rs/src/loader/python/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod channel;
mod context;
mod envelope;
mod node;
mod unlimited;
mod utils;

use crate::registry::Collect;

use super::{Loader, Plugin, PluginType};
use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Once};

#[derive(Clone)]
pub(crate) struct RegistryNodeParams {
    pub name: String,
    pub code: PyObject,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub exclusive: bool,
}

struct NodePlugin {
    local_key: u64,
    params: RegistryNodeParams,
}

impl NodePlugin {
    fn new(local_key: u64, params: RegistryNodeParams) -> NodePlugin {
        NodePlugin { local_key, params }
    }

    fn boxed(self) -> Box<dyn Plugin> {
        Box::new(self)
    }
}

impl Plugin for NodePlugin {
    fn submit(&self) {
        let params = self.params.clone();
        crate::node::NodeSlice::registry_local()
            .get(self.local_key)
            .insert(
                self.params.name.clone(),
                crate::node::NodeSlice {
                    cons: Box::new(move |name: String, args: &toml::value::Table| {
                        Box::new(node::PyNode::new(name, args, &params))
                    }),
                    info: flow_rs::node::NodeInfo {
                        inputs: self.params.inputs.clone(),
                        outputs: self.params.outputs.clone(),
                    },
                },
            );
    }
}

struct ResourcePlugin {
    local_key: u64,
    name: String,
    res: PyObject,
}

impl ResourcePlugin {
    fn boxed(self) -> Box<dyn Plugin> {
        Box::new(self)
    }
}

impl Plugin for ResourcePlugin {
    fn submit(&self) {
        let res = self.res.clone();
        crate::resource::ResourceSlice::registry_local()
            .get(self.local_key)
            .insert(
                self.name.clone(),
                crate::resource::ResourceSlice {
                    cons: Box::new(move |name: String, args: &toml::value::Table| {
                        let imp = Python::with_gil(|py| -> _ {
                            let pyargs = node::toml2dict(py, args)
                                .expect("convert toml to python dict fault");
                            match res.call1(py, (name.as_str(), pyargs)) {
                                Err(err) => {
                                    err.print(py);
                                    panic!("parse python code fault");
                                }
                                Ok(ret) => ret,
                            }
                        });
                        Arc::new(imp)
                    }),
                },
            );
    }
}

static ONCE_REGISTER: Once = Once::new();

struct PythonLoader;
const ERR_MSG: &str = "python plugin parse fault";

impl Loader for PythonLoader {
    fn load(
        &self,
        local_key: u64,
        module_path: &Path,
        plugin_path: &Path,
    ) -> Result<Vec<Box<dyn Plugin>>> {
        pyo3::prepare_freethreaded_python();
        let mut plugins = vec![];

        let module_name = path_to_module(module_path, plugin_path)?;

        Python::with_gil(|py| -> PyResult<_> {
            let module_path = module_path.display().to_string();
            let syspath: &PyList = py.import("sys")?.getattr("path")?.try_into()?;
            if !syspath
                .iter()
                .any(|path| module_path.as_str() == path.extract::<&str>().unwrap())
            {
                syspath.insert(0, module_path)?;
            }

            ONCE_REGISTER.call_once(|| {
                let module = py.import("megflow").expect("module megflow not found");
                utils::utils_register(module).expect("python utility functions register fault");
                envelope::envelope_register(module).expect("python envelope register fault");
            });

            py.import(module_name.as_str())?;

            let plugins_param: HashMap<String, Vec<&PyDict>> = py
                .import("megflow")?
                .getattr("collect")?
                .call0()?
                .extract()?;
            for plugin_param in &plugins_param["nodes"] {
                let name: String = plugin_param.get_item("name").expect(ERR_MSG).extract()?;
                let inputs: Vec<String> =
                    plugin_param.get_item("inputs").expect(ERR_MSG).extract()?;
                let outputs: Vec<String> =
                    plugin_param.get_item("outputs").expect(ERR_MSG).extract()?;
                let code: PyObject = plugin_param.get_item("code").expect(ERR_MSG).extract()?;
                let exclusive: bool = plugin_param
                    .get_item("exclusive")
                    .expect(ERR_MSG)
                    .extract()?;
                plugins.push(
                    NodePlugin::new(
                        local_key,
                        RegistryNodeParams {
                            name,
                            code,
                            inputs,
                            outputs,
                            exclusive,
                        },
                    )
                    .boxed(),
                );
            }
            for plugin_param in &plugins_param["resources"] {
                let name: String = plugin_param.get_item("name").expect(ERR_MSG).extract()?;
                let code: PyObject = plugin_param.get_item("code").expect(ERR_MSG).extract()?;
                plugins.push(
                    ResourcePlugin {
                        local_key,
                        name,
                        res: code,
                    }
                    .boxed(),
                )
            }
            Ok(())
        })?;

        Ok(plugins)
    }
}

crate::submit!(
    PluginType::Python,
    Box::new(PythonLoader {}) as Box<dyn Loader>
);

fn path_to_module(root: &Path, path: &Path) -> Result<String> {
    let abs_root = std::fs::canonicalize(root)?;
    let abs_path = std::fs::canonicalize(path)?;

    if abs_path.starts_with(&abs_root) && abs_path != abs_root {
        let relative = abs_path.strip_prefix(&abs_root)?;
        let relative = relative.to_str().unwrap().to_owned();
        if relative.ends_with(".py") {
            Ok(relative[..relative.len() - 3].replace("/", "."))
        } else if relative.ends_with('/') {
            Ok(relative[..relative.len() - 1].replace("/", "."))
        } else {
            Ok(relative.replace("/", "."))
        }
    } else {
        Err(anyhow::anyhow!("module not found"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_to_module() {
        let mut module_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        module_path.pop();
        module_path.push("flow-python/examples");
        let mut path = PathBuf::from(&module_path);
        path.push("logical_test/source.py");
        let ret = path_to_module(&module_path, &path);
        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), "logical_test.source".to_owned());

        path.pop();
        let ret = path_to_module(&module_path, &path);
        assert!(ret.is_ok());
        assert_eq!(ret.unwrap(), "logical_test".to_owned());

        path.pop();
        let ret = path_to_module(&module_path, &path);
        assert!(ret.is_err());

        path.pop();
        let ret = path_to_module(&module_path, &path);
        assert!(ret.is_err());
    }
}
