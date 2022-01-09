/**
 * \file flow-rs/src/loader/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
#[cfg(feature = "python")]
pub mod python;

use crate::registry::Collect;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

trait Plugin {
    fn submit(&self);
}

trait Loader: Send + Sync {
    fn load_from_file(
        &self,
        local_key: u64,
        module_path: Option<&Path>,
        plugin_path: &Path,
    ) -> Result<Vec<Box<dyn Plugin>>>;
    fn load_from_scope(&self, local_key: u64) -> Result<Vec<Box<dyn Plugin>>>;
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum PluginType {
    Python,
}

crate::collect!(PluginType, Box<dyn Loader>);

pub struct LoaderConfig {
    pub plugin_path: Option<PathBuf>,
    pub module_path: Option<PathBuf>,
    pub ty: PluginType,
}

use std::sync::Once;
static ONCE_LOAD_FROM_SCOPE: Once = Once::new();

pub(crate) fn load(local_key: u64, cfg: &LoaderConfig) -> Result<()> {
    let loader = <Box<dyn Loader>>::registry_global().get(&cfg.ty).unwrap();
    ONCE_LOAD_FROM_SCOPE.call_once(|| {
        for plugin in loader
            .load_from_scope(local_key)
            .expect("cannot load plugins from current scope")
        {
            plugin.submit();
        }
    });

    if let Some(plugin_path) = &cfg.plugin_path {
        for entry in fs::read_dir(&plugin_path)? {
            let entry = entry?;
            let pathbuf = entry.path();
            let path: &Path = pathbuf.as_ref();
            if let Some(path) = cfg.ty.check(path) {
                for plugin in loader.load_from_file(
                    local_key,
                    cfg.module_path.as_ref().map(|x| x.as_ref()),
                    path.as_path(),
                )? {
                    plugin.submit();
                }
            }
        }
    }

    Ok(())
}

impl PluginType {
    fn check(&self, path: &Path) -> Option<PathBuf> {
        match *self {
            PluginType::Python => PluginType::check_python(path),
        }
    }

    fn check_python(path: &Path) -> Option<PathBuf> {
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "py" {
                    return Some(path.to_path_buf());
                }
            }
            None
        } else {
            let mut buf = path.to_path_buf();
            buf.push("__init__.py");
            if buf.exists() {
                buf.pop();
                Some(buf)
            } else {
                None
            }
        }
    }
}
