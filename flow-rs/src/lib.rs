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
#[cfg(feature = "debug")]
mod debug;
pub mod envelope;
mod future;
pub mod graph;
#[cfg(feature = "python")]
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
    pub use super::Builder;
    /// Re-exports async_std as rt
    pub use async_std as rt;
    pub use flow_derive::*;
}

use anyhow::{anyhow, Result};
/// Re-exports async_std as rt
pub use async_std as rt;
#[doc(hidden)]
pub use ctor::*;
#[cfg(feature = "debug")]
pub use debug::Server as DebugServer;
pub use flow_derive::*;
use node::NodeInfo;
use prelude::MainGraph;
use registry::Collect;
use std::path::Path;

/// A builder to load graph with config
pub struct Builder {
    local_key: u64,
    template: String,
    dynamic: String,
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        let local_key = LOCAL_KEY.fetch_add(1, Ordering::Relaxed);
        registry::initialize(local_key);
        Builder {
            local_key,
            template: Default::default(),
            dynamic: Default::default(),
        }
    }

    pub fn load_plugins(self, cfg: loader::LoaderConfig) -> Self {
        loader::load(self.local_key, &cfg).unwrap();
        self
    }

    pub fn template_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        self.template = std::fs::read_to_string(path.as_ref())?;
        Ok(self)
    }

    pub fn dynamic_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        self.dynamic = std::fs::read_to_string(path.as_ref())?;
        Ok(self)
    }

    pub fn template(mut self, template: String) -> Self {
        self.template = template;
        self
    }

    pub fn dynamic(mut self, dynamic: String) -> Self {
        self.dynamic = dynamic;
        self
    }

    pub fn build(self) -> Result<MainGraph> {
        load_impl(
            self.local_key,
            config::parser::Parser::from_str(self.template.as_ref(), Some(self.dynamic.as_ref()))?,
        )
    }
}

use std::sync::atomic::{AtomicU64, Ordering};

lazy_static::lazy_static!(
    static ref LOCAL_KEY: AtomicU64 = AtomicU64::new(0);
);

fn load_impl(local_key: u64, config: config::presentation::Config) -> Result<graph::MainGraph> {
    // register subgraph info
    for cfg in &config.graphs {
        let info = NodeInfo {
            inputs: cfg.inputs.iter().map(|conn| conn.name.clone()).collect(),
            outputs: cfg.outputs.iter().map(|conn| conn.name.clone()).collect(),
        };
        graph::GraphSlice::registry_local().get(local_key).insert(
            cfg.name.clone(),
            graph::GraphSlice {
                cons: Box::new(move |_, _| Err(anyhow!("graph is not loaded"))),
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
                cons: Box::new(move |name, table| {
                    graph::Graph::load(
                        graph::context(name, cfg.name.clone(), local_key),
                        &cfg,
                        table,
                    )
                }),
                info,
            },
        );
    }

    // register shared nodes
    let ctx = graph::context("__GLOBAL__".to_owned(), "__GLOBAL__".to_owned(), local_key);
    let global_resources =
        resource::UniqueResourceCollection::new(ctx.local_key, ctx.id, &config.resources)
            .take_into_arc();
    for k in global_nodes_keys {
        let cfg = config.nodes.get(&k).unwrap();
        node::SharedProxy::registry_local().get(local_key).insert(
            cfg.entity.name.clone(),
            node::load_shared(cfg, &config, ctx.clone(), global_resources.clone())?,
        );
    }

    match graph::GraphSlice::registry_local()
        .get(local_key)
        .get(&config.main)
        .map(|slice| (slice.cons)(config.main.clone(), &Default::default()))
    {
        Some(ret) => ret.map(|g| MainGraph::new(g, ctx, global_resources)),
        _ => Err(anyhow!("graph {} is not exist", config.main)),
    }
}
