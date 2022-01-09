/**
 * \file flow-rs/src/config/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
pub mod graphviz;
mod insert;
pub mod interlayer;
pub mod table;

pub mod parser;
pub mod presentation;

use crate::node::{inputs, outputs};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub(crate) struct PortUtility {
    pub(crate) ty: interlayer::PortTy,
    pub(crate) mapping: fn(&str) -> String,
}
pub(crate) static MAPPING: &[PortUtility] = &[
    PortUtility {
        ty: interlayer::PortTy::Unit,
        mapping: |p| p.to_owned(),
    },
    PortUtility {
        ty: interlayer::PortTy::List,
        mapping: |p| format!("[{}]", p),
    },
    PortUtility {
        ty: interlayer::PortTy::Dict,
        mapping: |p| format!("{{{}}}", p),
    },
    PortUtility {
        ty: interlayer::PortTy::Dyn,
        mapping: |p| format!("dyn@{}", p),
    },
];

fn translate_node(
    local_key: u64,
    is_shared: bool,
    p: presentation::Node,
) -> Result<interlayer::Node> {
    let ty = p.entity.ty.split('|').next().unwrap().trim();
    let inputs = inputs(local_key, ty)?.into_iter().collect();
    let outputs = outputs(local_key, ty)?.into_iter().collect();
    Ok(interlayer::Node {
        entity: interlayer::Entity {
            name: p.entity.name,
            ty: p
                .entity
                .ty
                .split('|')
                .map(|x| x.trim().to_owned())
                .collect(),
            args: p.entity.args,
        },
        res: p.res,
        cloned: p.cloned,
        inputs,
        outputs,
        is_dyn: false,
        is_shared,
    })
}

fn translate_conn(
    p: presentation::Connection,
    nodes: &mut HashMap<String, interlayer::Node>,
    shared_nodes: &mut HashMap<String, interlayer::Node>,
) -> Result<interlayer::Connection> {
    let mut rx = vec![];
    let mut tx = vec![];

    if p.ports.is_empty() {
        return Err(anyhow!("encountered an unused connections"));
    }

    for port_s in p.ports {
        let ((n, p), tag) = interlayer::Port::parse(port_s.as_str())?;
        let mut parse = |node: &mut interlayer::Node| {
            let mut find = false;
            for utility in MAPPING {
                let p = (utility.mapping)(p);
                let port = interlayer::Port {
                    node_type: node.entity.ty.clone(),
                    node_name: node.entity.name.clone(),
                    port_name: p.clone(),
                    port_type: utility.ty,
                    port_tag: tag,
                };
                if node.inputs.contains(&p) {
                    rx.push(port);
                    find = true;
                    break;
                } else if node.outputs.contains(&p) {
                    tx.push(port);
                    find = true;
                    break;
                }
            }
            if !find {
                Err(anyhow!("unexpected port {}", port_s))
            } else {
                Ok(())
            }
        };
        if let Some(node) = nodes.get_mut(n) {
            parse(node)?;
        } else if let Some(node) = shared_nodes.get_mut(n) {
            parse(node)?;
        }
    }

    let dyn_rxn = rx.iter().filter(|rx| rx.is_dyn()).count();
    let dyn_txn = tx.iter().filter(|tx| tx.is_dyn()).count();

    if dyn_rxn > 0 {
        if tx.len() > 1 {
            return Err(anyhow!("dyn port shared with multiple subgraphs"));
        }
        if dyn_rxn > 1 {
            return Err(anyhow!(
                "the number of nodes sharing dyn rx port only could be equal to 1"
            ));
        }
        for p in &tx {
            let node = nodes
                .get_mut(&p.node_name)
                .expect("dyn share subgraph is not supported ");
            node.is_dyn = true;
        }
    }

    if dyn_txn > 0 {
        if rx.len() > 1 {
            return Err(anyhow!("dyn port shared with multiple subgraphs"));
        }
        for p in &rx {
            let node = nodes
                .get_mut(&p.node_name)
                .expect("dyn share subgraph is not supported ");
            node.is_dyn = true;
        }
    }

    Ok(interlayer::Connection { cap: p.cap, rx, tx })
}

pub fn translate_graph(
    local_key: u64,
    mut p: presentation::Graph,
    shared_nodes: &mut HashMap<String, interlayer::Node>,
) -> Result<interlayer::Graph> {
    let mut nodes = HashMap::new();
    let mut resources = HashMap::new();
    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut connections = HashMap::new();

    for node in std::mem::take(&mut p.nodes) {
        nodes.insert(
            node.entity.name.clone(),
            translate_node(local_key, false, node)?,
        );
    }
    for res in std::mem::take(&mut p.resources) {
        resources.insert(res.name.clone(), res);
    }

    for (i, conn) in std::mem::take(&mut p.connections).into_iter().enumerate() {
        let name = format!("__{}__", i);
        let mut conn = translate_conn(conn, &mut nodes, shared_nodes)?;
        insert::bcast(&name, &mut conn, &mut connections, &mut nodes);
        connections.insert(name, conn);
    }

    for mut input in std::mem::take(&mut p.inputs) {
        inputs.push(input.name.clone());
        insert::dyn_inp_trans(&mut input, &mut connections, &mut nodes, shared_nodes)?;
        let mut conn = translate_conn(input.conn, &mut nodes, shared_nodes)?;
        insert::bcast(&input.name, &mut conn, &mut connections, &mut nodes);
        connections.insert(input.name, conn);
    }

    for mut output in std::mem::take(&mut p.outputs) {
        outputs.push(output.name.clone());
        insert::dyn_out_trans(&mut output, &mut connections, &mut nodes, shared_nodes)?;
        let conn = translate_conn(output.conn, &mut nodes, shared_nodes)?;
        connections.insert(output.name, conn);
    }

    Ok(interlayer::Graph {
        is_shared: shared_nodes.values().any(|n| n.entity.ty.contains(&p.name)),
        name: p.name,
        resources,
        nodes,
        inputs,
        outputs,
        connections,
        global_res: vec![],
    })
}

pub fn translate_config(local_key: u64, p: presentation::Config) -> Result<interlayer::Config> {
    let mut graphs = vec![];
    let mut nodes = HashMap::new();
    let mut resources = HashMap::new();

    for node in p.nodes {
        nodes.insert(
            node.entity.name.clone(),
            translate_node(local_key, true, node)?,
        );
    }

    for graph in p.graphs {
        graphs.push(translate_graph(local_key, graph, &mut nodes)?);
    }

    for res in p.resources {
        resources.insert(res.name.clone(), res);
    }

    let mut cfg = interlayer::Config {
        graphs,
        resources,
        nodes,
        main: p.main,
    };
    insert::global_res(&mut cfg);

    Ok(cfg)
}
