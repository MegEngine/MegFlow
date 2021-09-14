/**
 * \file flow-rs/src/graph/subgraph.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::broker::BrokerClient;
use crate::prelude::*;
use crate::registry::Collect;
use crate::rt::task::JoinHandle;
use futures_util::{pin_mut, select, FutureExt};

impl Node for Graph {
    fn set_port(&mut self, port_name: &str, tag: Option<u64>, channel: &ChannelStorage) {
        let port_name = port_name.to_owned();
        let local_key = self.ctx.local_key;
        if let Some(chan) = self.conns.get_mut(&port_name) {
            chan.set(channel.clone());
            for port in chan.info().rx.iter().chain(chan.info().tx.iter()) {
                if let Some(nodes) = self.nodes.get_mut(&port.node_name) {
                    for node in nodes.get_mut() {
                        node.set_port(&port.port_name, tag, channel);
                    }
                } else {
                    let proxy = self
                        .shares
                        .entry(port.node_name.clone())
                        .or_insert_with(|| {
                            SharedProxy::registry_local()
                                .get(local_key)
                                .get(&port.node_name)
                                .unwrap_or_else(|| {
                                    panic!("global node not found {}", port.node_name)
                                })
                                .as_ref()
                                .clone()
                        });
                    proxy.set_port(&port.port_name, channel);
                }
            }
        } else {
            unreachable!("unexpected port {} for graph {}", port_name, self.ctx.name)
        }
    }

    fn set_port_dynamic(&mut self, _: u64, _: &str, _: String, _: usize, _: BrokerClient) {
        unimplemented!()
    }

    fn close(&mut self) {
        for input in &self.inputs {
            self.conns[input].get().close();
        }
    }

    fn is_allinp_closed(&self) -> bool {
        let mut is_closed = true;
        for input in &self.inputs {
            is_closed = is_closed && self.conns[input].get().is_closed();
        }
        is_closed
    }
}

impl Actor for Graph {
    fn start(
        mut self: Box<Self>,
        context: Context,
        resource: ResourceCollection,
    ) -> JoinHandle<()> {
        flow_rs::rt::task::spawn(async move {
            let wait_ctx = context.wait().fuse();
            let wait_graph = self.as_mut().start(Some(resource)).fuse();
            pin_mut!(wait_ctx, wait_graph);
            loop {
                select! {
                    _ = wait_ctx => {
                        self.close()
                    }
                    _ = wait_graph => context.close(),
                    complete => break,
                }
            }
        })
    }
}
