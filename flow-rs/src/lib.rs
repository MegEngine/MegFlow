//! MegFlow

/**
 * \file flow-rs/src/lib.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
extern crate self as flow_rs;

#[doc(hidden)]
pub mod broker;
pub mod channel;
mod config;
mod debug;
pub mod envelope;
pub mod graph;
pub mod helper;
pub mod loader;
pub mod node;
#[doc(hidden)]
pub mod registry;
pub mod resource;
#[cfg(test)]
pub mod sandbox;

pub mod prelude {
    #[doc(hidden)]
    pub use super::broker::*;
    pub use super::channel::*;
    pub use super::envelope::*;
    pub use super::graph::*;
    pub use super::node::*;
    pub use super::registry::*;
    pub use super::resource::*;
    pub use super::{load, load_single_graph};
    /// Re-exports async_std as rt
    pub use async_std as rt;
    pub use flow_derive::*;
}

use anyhow::{anyhow, Result};
/// Re-exports async_std as rt
pub use async_std as rt;
#[doc(hidden)]
pub use ctor::*;
pub use debug::Server as DebugServer;
pub use flow_derive::*;
use node::NodeInfo;
use registry::Collect;

/// A function to load graph with config
pub fn load(option: Option<loader::LoaderConfig>, config: &str) -> Result<graph::Graph> {
    load_impl(option, toml::from_str(config)?)
}
/// A convenient function to load single graph
pub fn load_single_graph(
    option: Option<loader::LoaderConfig>,
    config: &str,
) -> Result<graph::Graph> {
    let config: config::presentation::Graph = toml::from_str(config)?;
    let config = config::presentation::Config {
        main: config.name.clone(),
        graphs: vec![config],
        nodes: vec![],
        resources: vec![],
    };
    load_impl(option, config)
}

use std::sync::atomic::{AtomicU64, Ordering};

lazy_static::lazy_static!(
    static ref LOCAL_KEY: AtomicU64 = AtomicU64::new(0);
);

fn load_impl(
    option: Option<loader::LoaderConfig>,
    config: config::presentation::Config,
) -> Result<graph::Graph> {
    // init graph local resources
    let local_key = LOCAL_KEY.fetch_add(1, Ordering::Relaxed);
    registry::initialize(local_key);
    // load plugins
    if let Some(option) = option {
        loader::load(local_key, &option).unwrap();
    }
    // register global resources
    for res in &config.resources {
        let mut global_res = resource::GLOBAL_RESOURCES.write().unwrap();
        global_res.get_mut(&local_key).unwrap().insert(
            res.name.clone(),
            res.ty.as_str(),
            &res.args,
        );
    }
    // register subgraph info
    for cfg in &config.graphs {
        let info = NodeInfo {
            inputs: cfg.inputs.iter().map(|conn| conn.name.clone()).collect(),
            outputs: cfg.outputs.iter().map(|conn| conn.name.clone()).collect(),
        };
        graph::GraphSlice::registry_local().get(local_key).insert(
            cfg.name.clone(),
            graph::GraphSlice {
                cons: Box::new(move |_| Err(anyhow!("graph is not loaded"))),
                info,
            },
        );
    }
    let global_nodes_keys: Vec<_> = config
        .nodes
        .iter()
        .map(|n| &n.entity.name)
        .cloned()
        .collect();

    let config = config::translate_config(local_key, config)?;
    config::graphviz::dump(&config)?;

    // update graph constructor
    for cfg in &config.graphs {
        let cfg = cfg.clone();
        let info = NodeInfo {
            inputs: cfg.inputs.clone(),
            outputs: cfg.outputs.clone(),
        };
        graph::GraphSlice::registry_local().get(local_key).insert(
            cfg.name.clone(),
            graph::GraphSlice {
                cons: Box::new(move |name| {
                    graph::Graph::load(graph::context(name, cfg.name.clone(), local_key), &cfg)
                }),
                info,
            },
        );
    }

    // register shared nodes
    let ctx = graph::context("__GLOBAL__".to_owned(), "__GLOBAL__".to_owned(), local_key);
    for k in global_nodes_keys {
        let cfg = config.nodes.get(&k).unwrap();
        let global_res = resource::GLOBAL_RESOURCES.read().unwrap();
        let res = global_res.get(&local_key).unwrap().filter(
            cfg.res
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        node::SharedProxy::registry_local().get(local_key).insert(
            cfg.entity.name.clone(),
            node::load_shared(cfg, ctx.clone(), res)?,
        );
    }

    match graph::GraphSlice::registry_local()
        .get(local_key)
        .get(&config.main)
        .map(|slice| (slice.cons)(config.main.clone()))
    {
        Some(mut ret) => {
            let _ = ret.as_mut().map(|g| g.mark_main());
            ret
        }
        _ => Err(anyhow!("graph {} is not exist", config.main)),
    }
}
