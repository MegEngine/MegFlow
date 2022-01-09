/**
 * \file flow-rs/src/graph/mod.rs
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
#[cfg(feature = "debug")]
mod debug;
mod node;
mod subgraph;

use crate::broker::Broker;
use crate::config::interlayer as config;
use crate::config::table::merge_table;
use crate::prelude::*;
use crate::rt::task::JoinHandle;
use anyhow::{anyhow, Result};
use channel::*;
pub use context::*;
use futures_util::{pin_mut, select, FutureExt};
use node::AnyNode;
use std::collections::HashMap;
use toml::value::Table;

pub(crate) struct GraphSlice {
    pub cons: Box<dyn Fn(String, &Table) -> Result<Graph> + Send + Sync>,
    pub info: NodeInfo,
}
crate::collect!(String, GraphSlice);

/// Represents a graph with nodes and connections, which implement `Node` and `Actor`.
pub struct MainGraph {
    graph: Graph,
    global_resources: ResourceCollection,
    global_ctx: Context,
}

impl MainGraph {
    pub(crate) fn new(
        mut graph: Graph,
        global_ctx: Context,
        global_resources: ResourceCollection,
    ) -> MainGraph {
        let mut v = vec![];
        for name in &graph.inputs {
            v.push((name.clone(), graph.conns[name].make()));
        }
        for name in &graph.outputs {
            v.push((name.clone(), graph.conns[name].make()));
        }

        for (name, conn) in v {
            graph.set_port(name.as_str(), None, &conn);
        }
        MainGraph {
            graph,
            global_ctx,
            global_resources,
        }
    }
    /// Get an input port from the graph by name
    pub fn input(&self, name: &str) -> Option<Sender> {
        self.graph.conns.get(name).map(|x| x.get().sender())
    }
    /// Get an output port from the graph by name
    pub fn output(&self, name: &str) -> Option<Receiver> {
        self.graph.conns.get(name).map(|x| x.get().receiver())
    }
    /// Get inputs names
    pub fn input_names(&self) -> Vec<&str> {
        self.graph.inputs.iter().map(|x| x.as_str()).collect()
    }
    /// Get outputs names
    pub fn output_names(&self) -> Vec<&str> {
        self.graph.outputs.iter().map(|x| x.as_str()).collect()
    }
    /// Stop the graph, it is equivalent to drop all inputs of the graph
    pub fn stop(mut self) {
        self.graph.close()
    }
    /// Run the graph, and return a `JoinHandle`, which can be cancel or wait
    pub fn start(&mut self) -> JoinHandle<Result<()>> {
        let handle = self
            .graph
            .start(Some(std::mem::take(&mut self.global_resources)));
        let global_ctx = self.global_ctx.clone();
        crate::rt::task::spawn(async move {
            handle.await?;
            SharedProxy::registry_local()
                .get(global_ctx.local_key)
                .for_each(|proxy| proxy.close());
            global_ctx.close();

            let handles = crate::node::SharedHandle::registry_local()
                .get(global_ctx.local_key)
                .to_vec();
            for handle in handles {
                let handle =
                    std::sync::Arc::try_unwrap(handle).unwrap_or_else(|_| panic!("internal error"));
                handle.0.await?;
            }
            // clear graph local resources
            finalize(global_ctx.local_key);
            Ok(())
        })
    }
}

pub(crate) struct Graph {
    nodes: HashMap<String, AnyNode>,
    conns: HashMap<String, AnyChannel>,
    inputs: Vec<String>,
    outputs: Vec<String>,
    broker: Broker,
    shares: HashMap<String, SharedProxy>,
    ctx: Context,
    resources: UniqueResourceCollection,
    is_shared: bool,
}

impl Graph {
    pub(crate) fn load(ctx: Context, config: &config::Graph, args: &Table) -> Result<Graph> {
        let mut nodes = HashMap::new();
        let mut conns = HashMap::new();
        let mut broker = Broker::new();
        let mut shares = HashMap::new();

        // global
        let resources = UniqueResourceCollection::new(ctx.local_key, ctx.id, &config.resources);

        for (name, cfg) in &config.nodes {
            if !cfg.is_dyn {
                nodes.insert(
                    name.clone(),
                    AnyNode::new(ctx.local_key, cfg.clone(), args.clone())?,
                );
            }
        }

        for (k, cfg) in &config.connections {
            let dyn_rxn = cfg.rx.iter().filter(|rx| rx.is_dyn()).count();
            let dyn_txn = cfg.tx.iter().filter(|tx| tx.is_dyn()).count();

            if !config.inputs.contains(k) && !config.outputs.contains(k) {
                if dyn_rxn > 0 && dyn_txn > 0 {
                    return Err(anyhow!("rx & tx of channel both are dyn"));
                }
                if (dyn_rxn > 0 && cfg.tx.len() != 1) || (dyn_txn > 0 && cfg.rx.len() != 1) {
                    return Err(anyhow!("dyn port shared with multiple subgraphs"));
                }
                if (dyn_rxn != 0 && dyn_rxn != cfg.rx.len())
                    || (dyn_txn != 0 && dyn_txn != cfg.tx.len())
                {
                    return Err(anyhow!("dyn port shared with multiple subgraphs"));
                }

                if dyn_rxn > 0 || dyn_txn > 0 {
                    let subgraph = cfg
                        .rx
                        .iter()
                        .chain(cfg.tx.iter())
                        .find(|p| !p.is_dyn())
                        .unwrap();
                    for port in cfg.rx.iter().chain(cfg.tx.iter()) {
                        if port.is_dyn() {
                            // add dyn subgraph resources into trigger node
                            let subgraph_cfg = config.nodes.get(&subgraph.node_name).unwrap();
                            let subgraph_args = subgraph_cfg.entity.args.clone();
                            let mut res = subgraph_cfg.res.clone();
                            let nodes = nodes.get_mut(&port.node_name).unwrap();
                            let info = nodes.info_mut();
                            info.res.append(&mut res);
                            for node in nodes.get_mut().iter_mut() {
                                let mut clients = vec![];
                                for topic in &subgraph.node_type {
                                    clients.push(broker.subscribe(topic.trim().to_owned()));
                                }
                                node.set_port_dynamic(
                                    &port.port_name,
                                    crate::node::DynPortsConfig {
                                        local_key: ctx.local_key,
                                        target: subgraph.port_name.clone(),
                                        cap: cfg.cap,
                                        brokers: clients,
                                        args: merge_table(args.clone(), subgraph_args.clone()),
                                    },
                                );
                            }
                        }
                    }
                } else {
                    let mut channel = AnyChannel::new(cfg)?;
                    channel.set(channel.make());
                    let info = channel.info();
                    for port in info.rx.iter().chain(info.tx.iter()) {
                        if let Some(nodes) = nodes.get_mut(&port.node_name) {
                            for node in nodes.get_mut().iter_mut() {
                                node.set_port(&port.port_name, port.port_tag, channel.get());
                            }
                        } else {
                            if config.is_shared {
                                return Err(anyhow!("nested shared graph is not support"));
                            }
                            let shared_proxy =
                                shares.entry(port.node_name.clone()).or_insert_with(|| {
                                    SharedProxy::registry_local()
                                        .get(ctx.local_key)
                                        .get(&port.node_name)
                                        .unwrap_or_else(|| {
                                            panic!("unexpected node {}", port.node_name)
                                        })
                                        .as_ref()
                                        .clone()
                                });
                            shared_proxy.set_port(&port.port_name, channel.get());
                        }
                    }
                    conns.insert(k.clone(), channel);
                }
            } else {
                if dyn_rxn > 0 || dyn_txn > 0 {
                    return Err(anyhow!("dyn inputs or outputs of graph"));
                }
                for port in cfg.rx.iter().chain(cfg.tx.iter()) {
                    if let Some(node) = config.nodes.get(&port.node_name) {
                        if node.is_dyn {
                            return Err(anyhow!(
                                "dyn nodes connect with inputs or outputs of graph"
                            ));
                        }
                    }
                }
                let channel = AnyChannel::new(cfg)?;
                conns.insert(k.clone(), channel);
            }
        }

        Ok(Graph {
            ctx,
            resources,
            nodes,
            conns,
            broker,
            inputs: config.inputs.clone(),
            outputs: config.outputs.clone(),
            shares,
            is_shared: config.is_shared,
        })
    }

    pub(crate) fn start(&mut self, resource: Option<ResourceCollection>) -> JoinHandle<Result<()>> {
        let ext_resource = resource.unwrap_or_else(|| {
            UniqueResourceCollection::new(self.ctx.local_key, self.ctx.id, &Default::default())
                .take_into_arc()
        });
        let in_resource = std::mem::take(&mut self.resources);
        let shares = std::mem::take(&mut self.shares);
        for (_, shared) in shares {
            shared.build();
        }
        let mut handles = vec![self.broker.run()];
        let mut alone_tasks = vec![
            #[cfg(feature = "debug")]
            self.dmon(),
        ];
        let nodes: Vec<_> = self
            .nodes
            .values_mut()
            .map(|node| {
                let is_alone = node.info().inputs.is_empty() && node.info().outputs.is_empty();
                let res_names: Vec<_> = node.info().res.to_vec();
                (is_alone, res_names, node.get_into())
            })
            .collect();

        let context = self.ctx.clone();
        let inputs: Vec<_> = self
            .inputs
            .iter()
            .map(|name| self.conns[name].get().clone())
            .collect();
        let outputs: Vec<_> = self
            .outputs
            .iter()
            .map(|name| self.conns[name].get().clone())
            .collect();
        let direct_term = inputs
            .iter()
            .map(|input| input.sender_count())
            .sum::<usize>()
            == 0;

        let handle = crate::rt::task::spawn(async move {
            let res = ext_resource.chain(in_resource).await;
            for (is_alone, res_names, nodes) in nodes {
                for node in nodes {
                    let res = res.filter(
                        res_names
                            .iter()
                            .map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    );
                    if is_alone {
                        alone_tasks.push(node.start(context.clone(), res));
                    } else {
                        handles.push(node.start(context.clone(), res));
                    }
                }
            }

            let wait_ctx = context.wait().fuse();
            let wait_tasks = futures_util::future::try_join_all(handles).fuse();
            let mut wait_i =
                futures_util::future::join_all(inputs.iter().map(|conn| conn.wait_rx_closed()))
                    .fuse();
            let mut wait_o =
                futures_util::future::join_all(outputs.iter().map(|conn| conn.wait_tx_closed()))
                    .fuse();
            pin_mut!(wait_ctx, wait_tasks);

            let mut state = 0;
            let mut cb = || {
                state += 1;
                if state == 3 {
                    context.close();
                }
            };

            let mut ret = Ok(());
            loop {
                select! {
                    _ = wait_ctx => {
                        for input in &inputs {
                            input.close();
                        }
                    }
                    task_ret = wait_tasks => {
                        match task_ret {
                            Ok(_) => cb(),
                            err => {
                                ret = err.map(|_| ());
                                context.close()
                            },
                        }
                    },
                    _ = wait_o => {
                        cb()
                    },
                    _ = wait_i => {
                        cb()
                    }
                    complete => {
                        break;
                    },
                }
            }

            futures_util::future::try_join_all(alone_tasks).await?;
            ret
        });

        if direct_term {
            self.close();
        }

        handle
    }
}
