/**
 * \file flow-rs/src/node/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod bcast;
mod demux;
mod noop;
mod port;
mod reorder;
mod shared;
mod transform;

use crate::channel::ChannelStorage;
use crate::graph::Context;
use crate::graph::GraphSlice;
use crate::registry::Collect;
use crate::resource::ResourceCollection;
use crate::rt::task::JoinHandle;
use anyhow::{anyhow, Result};
pub use port::*;
pub(crate) use shared::*;
use std::collections::BTreeSet;
use toml::value::Table;

#[doc(hidden)]
pub struct NodeSlice {
    pub cons: Box<dyn Fn(String, &Table) -> Box<dyn Actor> + Send + Sync>,
    pub info: NodeInfo,
}
#[doc(hidden)]
#[derive(Clone)]
pub struct NodeInfo {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}
crate::collect!(String, NodeSlice);
/// Trait for interactiving with graph, which can be derived by `#[derive(Node)]`.
pub trait Node {
    fn set_port(&mut self, port_name: &str, tag: Option<u64>, channel: &ChannelStorage);
    fn set_port_dynamic(&mut self, port_name: &str, config: DynPortsConfig);
    fn close(&mut self);
    fn is_allinp_closed(&self) -> bool;
}

/// Trait for interactiving with schedule, which can be derived by `#[derive(Actor)]`.
pub trait Actor: Node + Send + 'static {
    /// Run the actor
    fn start(self: Box<Self>, _: Context, _: ResourceCollection) -> JoinHandle<Result<()>> {
        unimplemented!()
    }
}

pub(crate) fn load_static(
    local_key: u64,
    config: &crate::config::interlayer::Node,
) -> Result<Vec<Box<dyn Actor>>> {
    if config.entity.ty.len() > 1 {
        return Err(anyhow!("static subgraph/node dont support dyn type"));
    }
    let ty = config.entity.ty.first().unwrap();
    if let Some(node) = NodeSlice::registry_local().get(local_key).get(ty) {
        Ok((0..config.cloned.unwrap_or(1))
            .into_iter()
            .map(|_| (node.cons)(config.entity.name.clone(), &config.entity.args))
            .collect())
    } else if let Some(graph) = GraphSlice::registry_local().get(local_key).get(ty) {
        (0..config.cloned.unwrap_or(1))
            .into_iter()
            .map(|_| (graph.cons)(config.entity.name.clone(), &config.entity.args))
            .map(|g| g.map(|g| Box::new(g) as Box<dyn Actor>))
            .collect()
    } else {
        Err(anyhow!("unexpected node type {:?}", config.entity.ty))
    }
}

pub(crate) fn inputs(local_key: u64, ty: &str) -> Result<BTreeSet<String>> {
    if let Some(node) = NodeSlice::registry_local().get(local_key).get(ty) {
        Ok(node.info.inputs.iter().cloned().collect())
    } else if let Some(graph) = GraphSlice::registry_local().get(local_key).get(ty) {
        Ok(graph.info.inputs.iter().cloned().collect())
    } else {
        Err(anyhow!("unexpected node type [{}]", ty))
    }
}

pub(crate) fn outputs(local_key: u64, ty: &str) -> Result<BTreeSet<String>> {
    if let Some(node) = NodeSlice::registry_local().get(local_key).get(ty) {
        Ok(node.info.outputs.iter().cloned().collect())
    } else if let Some(graph) = GraphSlice::registry_local().get(local_key).get(ty) {
        Ok(graph.info.outputs.iter().cloned().collect())
    } else {
        Err(anyhow!("unexpected node type [{}]", ty))
    }
}
