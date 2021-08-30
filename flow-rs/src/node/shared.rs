/**
 * \file flow-rs/src/node/shared.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::load;
use crate::prelude::*;
use crate::rt::channel::{unbounded, Receiver as ReceiverT, Sender as SenderT};
use crate::rt::sync::RwLock;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone)]
struct SharedConns {
    inputs: HashMap<String, Receiver>,
    outputs: HashMap<String, Sender>,
}

#[derive(Node)]
struct Shared {
    nodes: Vec<Box<dyn Actor>>,
    rx: ReceiverT<SharedConns>,
    inputs: HashMap<String, Arc<Sender>>,
    outputs: HashMap<String, Receiver>,
}

pub struct SharedStopNotify(pub oneshot::Receiver<rt::task::JoinHandle<()>>);
// is safe because we visit it only in main thread
unsafe impl Sync for SharedStopNotify {}
crate::collect!(String, SharedStopNotify);

#[derive(Clone)]
pub struct SharedProxy {
    ty: String,
    conns: SharedConns,
    tx: SenderT<SharedConns>,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl Shared {
    pub fn new(
        local_key: u64,
        rx: ReceiverT<SharedConns>,
        cfg: &crate::config::interlayer::Node,
    ) -> Result<Shared> {
        let mut nodes = load(local_key, cfg)?;
        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();

        for input in super::inputs(local_key, &cfg.entity.ty)? {
            let chan = ChannelStorage::unbound();
            for node in &mut nodes {
                node.set_port(input.as_str(), None, &chan);
            }
            inputs.insert(input.clone(), Arc::new(chan.sender()));
        }

        for output in super::outputs(local_key, &cfg.entity.ty)? {
            let chan = ChannelStorage::unbound();
            for node in &mut nodes {
                node.set_port(output.as_str(), None, &chan);
            }
            outputs.insert(output.clone(), chan.receiver());
        }

        Ok(Shared {
            nodes,
            rx,
            inputs,
            outputs,
        })
    }
    pub fn boxed(self) -> Box<Shared> {
        Box::new(self)
    }
}

impl Actor for Shared {
    fn start(
        mut self: Box<Self>,
        ctx: Context,
        resources: ResourceCollection,
    ) -> rt::task::JoinHandle<()> {
        let (s1, r1) = unbounded::<u64>();
        let mut s2s = vec![];
        let mut r2s = vec![];
        for _ in 0..self.outputs.len() {
            let (s2, r2) = unbounded::<u64>();
            s2s.push(s2);
            r2s.push(r2);
        }
        let inputs = self.inputs.clone();
        if !inputs.is_empty() {
            // close in order
            crate::rt::task::spawn(async move {
                while let Ok(id) = r1.recv().await {
                    for input in inputs.values() {
                        input.send_any(DummyEnvelope {}.seal()).await.ok();
                    }
                    for s2 in &s2s {
                        s2.send(id).await.ok();
                    }
                }
            });
        }

        let mut outputs: HashMap<_, Arc<RwLock<HashMap<_, Sender>>>> = HashMap::new();
        for (k, input) in std::mem::take(&mut self.outputs) {
            let demux = Arc::new(RwLock::new(HashMap::new()));
            outputs.insert(k, demux.clone());
            let r2 = r2s.pop().unwrap();
            crate::rt::task::spawn(async move {
                loop {
                    match input.recv_any().await {
                        Ok(msg) => {
                            let outputs = demux.read().await;
                            let output = outputs
                                .get(
                                    msg.info()
                                        .transfer_addr
                                        .as_ref()
                                        .expect("lost transfer_address"),
                                )
                                .expect("unexpected transfer_address");
                            output.send_any(msg).await.ok();
                        }
                        _ if !input.is_closed() => {
                            if let Ok(id) = r2.recv().await {
                                demux.write().await.remove(&id);
                            }
                        }
                        _ => break,
                    }
                }
            });
        }

        let mut handles = vec![];
        for node in std::mem::take(&mut self.nodes) {
            handles.push(node.start(ctx.clone(), resources.clone()));
        }

        if !self.inputs.is_empty() || !self.outputs.is_empty() {
            crate::rt::task::spawn(async move {
                let mut counter = 0;
                while let Ok(mut msg) = self.rx.recv().await {
                    let closed_n = Arc::new(AtomicUsize::new(self.inputs.len()));
                    let id = counter;
                    for (k, output) in self.inputs.iter_mut() {
                        let input = msg.inputs.remove(k).expect("shared port match fault");
                        let output = output.clone();
                        let s1 = s1.clone();
                        let closed_n = closed_n.clone();
                        crate::rt::task::spawn(async move {
                            loop {
                                while let Ok(mut envelope) = input.recv_any().await {
                                    envelope.info_mut().transfer_addr = Some(id);
                                    output.send_any(envelope).await.ok();
                                }
                                if input.is_closed() {
                                    break;
                                } else {
                                    // todo: fix nested shared subgraph
                                    unreachable!("nested shared subgraph is not supported");
                                }
                            }
                            if 1 == closed_n.fetch_sub(1, Ordering::Relaxed) {
                                s1.send(id).await.ok();
                            }
                        });
                    }
                    for (k, v) in outputs.iter() {
                        let output = msg.outputs.remove(k).expect("shared port match fault");
                        v.write().await.insert(id, output);
                    }
                    counter += 1;
                }
            });
        }

        crate::rt::task::spawn(async move {
            futures_util::future::join_all(handles).await;
        })
    }
}

impl SharedProxy {
    fn new(
        local_key: u64,
        tx: SenderT<SharedConns>,
        cfg: &crate::config::interlayer::Node,
    ) -> Result<SharedProxy> {
        let inputs: Vec<_> = inputs(local_key, &cfg.entity.ty)?.into_iter().collect();
        let outputs: Vec<_> = outputs(local_key, &cfg.entity.ty)?.into_iter().collect();
        Ok(SharedProxy {
            ty: cfg.entity.ty.clone(),
            tx,
            conns: SharedConns {
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            },
            inputs,
            outputs,
        })
    }

    pub fn set_port(&mut self, port_name: &str, chan: &ChannelStorage) {
        if self.inputs.iter().any(|p| p == port_name) {
            self.conns
                .inputs
                .insert(port_name.to_owned(), chan.receiver());
        } else if self.outputs.iter().any(|p| p == port_name) {
            self.conns
                .outputs
                .insert(port_name.to_owned(), chan.sender());
        } else {
            unreachable!()
        }
    }

    pub fn build(self) {
        self.tx.try_send(self.conns).ok();
    }

    pub fn close(&self) {
        self.tx.close();
    }
}

crate::collect!(String, SharedProxy);

pub fn load_shared(
    cfg: &crate::config::interlayer::Node,
    ctx: Context,
    resources: ResourceCollection,
) -> Result<SharedProxy> {
    let local_key = ctx.local_key;
    let (s, r) = unbounded();
    let shared = Shared::new(ctx.local_key, r, cfg)?.boxed();
    let handle = shared.start(ctx, resources);
    let (s2, r2) = oneshot::channel();
    s2.send(handle).ok();
    SharedStopNotify::registry_local()
        .get(local_key)
        .insert(cfg.entity.name.clone(), SharedStopNotify(r2));
    SharedProxy::new(local_key, s, cfg)
}
