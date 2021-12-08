/**
 * \file flow-rs/src/sandbox.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::broker::Broker;
use crate::registry::Collect;
use crate::resource::*;
use crate::rt::task::JoinHandle;
use crate::{
    channel::{Receiver, Sender},
    config::interlayer,
    graph::context,
    node::{inputs, load_static, outputs, Actor},
    prelude::ChannelStorage,
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Once;

static ONCE_INIT_I: Once = Once::new();
static ONCE_INIT_O: Once = Once::new();

pub struct Sandbox {
    local_key: u64,
    ty: String,
    actor: Box<dyn Actor>,
    broker: Broker,
    inputs: HashMap<String, ChannelStorage>,
    outputs: HashMap<String, ChannelStorage>,
}

impl Sandbox {
    pub fn pure(ty: &str) -> Result<Sandbox> {
        Sandbox::new(ty, Default::default(), vec![])
    }
    pub fn with_args(ty: &str, args: toml::value::Table) -> Result<Sandbox> {
        Sandbox::new(ty, args, vec![])
    }
    pub fn new(ty: &str, args: toml::value::Table, res: Vec<String>) -> Result<Sandbox> {
        let local_key = crate::LOCAL_KEY.fetch_add(1, Ordering::Relaxed);
        crate::registry::initialize(local_key);

        let config = interlayer::Node {
            entity: interlayer::Entity {
                name: format!("sandbox-{}", ty),
                ty: vec![ty.to_owned()],
                args,
            },
            res,
            cloned: None,
            inputs: Default::default(),
            outputs: Default::default(),
            is_dyn: false,
            is_shared: false,
        };
        let inputs_name: Vec<String> = inputs(local_key, ty)?.into_iter().collect();
        let outputs_name: Vec<String> = outputs(local_key, ty)?.into_iter().collect();
        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();
        let mut actor = load_static(local_key, &config)?.pop().unwrap();
        let mut broker = Broker::new();

        if inputs_name.iter().any(|x| x.starts_with("dyn")) {
            ONCE_INIT_I.call_once(|| {
                graph_register(local_key, "NoopProducer", "out");
            });
        }

        if outputs_name.iter().any(|x| x.starts_with("dyn")) {
            ONCE_INIT_O.call_once(|| {
                graph_register(local_key, "NoopConsumer", "inp");
            });
        }

        for input in &inputs_name {
            if !input.starts_with("dyn") {
                let chan = ChannelStorage::unbound();
                actor.set_port(input, Some(0), &chan);
                let mut input = input.as_str();
                if input.starts_with('[') || input.starts_with('{') {
                    input = &input[1..input.len() - 1];
                }
                inputs.insert(input.to_owned(), chan);
            } else {
                let client = broker.subscribe("_NoopProducer_".to_owned());
                actor.set_port_dynamic(local_key, input, "out".to_owned(), 16, vec![client]);
            }
        }
        for output in &outputs_name {
            if !output.starts_with("dyn") {
                let chan = ChannelStorage::unbound();
                actor.set_port(output, Some(0), &chan);
                let mut output = output.as_str();
                if output.starts_with('[') || output.starts_with('{') {
                    output = &output[1..output.len() - 1];
                }
                outputs.insert(output.to_owned(), chan);
            } else {
                let client = broker.subscribe("_NoopConsumer_".to_owned());
                actor.set_port_dynamic(local_key, output, "inp".to_owned(), 16, vec![client]);
            }
        }
        Ok(Sandbox {
            local_key,
            ty: ty.to_owned(),
            broker,
            actor,
            outputs,
            inputs,
        })
    }

    pub fn input(&self, name: &str) -> Option<Sender> {
        self.inputs.get(name).map(|x| x.sender())
    }

    pub fn output(&self, name: &str) -> Option<Receiver> {
        self.outputs.get(name).map(|x| x.receiver())
    }

    pub fn start(mut self) -> JoinHandle<()> {
        let local_key = self.local_key;
        let ctx = context("Sandbox".to_owned(), self.ty.clone(), local_key);
        crate::rt::task::spawn(async move {
            futures_util::join!(
                self.actor.start(
                    ctx.clone(),
                    UniqueResourceCollection::new(ctx.local_key, ctx.id, &Default::default())
                        .take_into_arc()
                ),
                self.broker.run()
            );
            crate::registry::finalize(local_key);
        })
    }
}

fn graph_register(local_key: u64, ty: &str, port_name: &str) {
    let inputs = if port_name == "inp" {
        vec![port_name.to_owned()]
    } else {
        vec![]
    };
    let outputs = if port_name == "out" {
        vec![port_name.to_owned()]
    } else {
        vec![]
    };
    let node = interlayer::Node {
        entity: interlayer::Entity {
            name: "noop".to_owned(),
            ty: vec![ty.to_owned()],
            args: Default::default(),
        },
        cloned: None,
        inputs: inputs.iter().cloned().collect(),
        outputs: outputs.iter().cloned().collect(),
        is_dyn: false,
        is_shared: false,
        res: vec![],
    };
    let port = interlayer::Port {
        node_type: vec![ty.to_owned()],
        node_name: "noop".to_owned(),
        port_type: interlayer::PortTy::Unit,
        port_tag: None,
        port_name: port_name.to_owned(),
    };
    let conn = interlayer::Connection {
        cap: 16,
        tx: inputs.iter().map(|_| port.clone()).collect(),
        rx: outputs.iter().map(|_| port.clone()).collect(),
    };
    let cfg = interlayer::Graph {
        name: format!("_{}_", ty),
        nodes: vec![("noop".to_owned(), node)].into_iter().collect(),
        inputs: inputs.clone(),
        outputs: outputs.clone(),
        connections: vec![(port_name.to_owned(), conn)].into_iter().collect(),
        resources: Default::default(),
        is_shared: false,
        global_res: vec![],
    };
    let info = crate::node::NodeInfo {
        inputs: cfg.inputs.clone(),
        outputs: cfg.outputs.clone(),
    };
    crate::graph::GraphSlice::registry_local()
        .get(local_key)
        .insert(
            cfg.name.clone(),
            crate::graph::GraphSlice {
                cons: Box::new(move |name| {
                    crate::graph::Graph::load(context(name, cfg.name.clone(), local_key), &cfg)
                }),
                info,
            },
        );
}
