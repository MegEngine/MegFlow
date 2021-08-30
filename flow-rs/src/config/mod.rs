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
pub mod interlayer;
pub mod presentation;

use crate::node::{inputs, outputs};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

struct PortUtility {
    ty: interlayer::PortTy,
    mapping: fn(&str) -> String,
}
static MAPPING: &[PortUtility] = &[
    PortUtility {
        ty: interlayer::PortTy::Unit,
        mapping: |p| p.to_owned(),
    },
    PortUtility {
        ty: interlayer::PortTy::List,
        mapping: |p| format!("[{}]", p),
    },
    PortUtility {
        ty: interlayer::PortTy::Dyn,
        mapping: |p| format!("dyn@{}", p),
    },
];

fn translate_node(local_key: u64, p: presentation::Node) -> Result<interlayer::Node> {
    let inputs = inputs(local_key, &p.entity.ty)?
        .into_iter()
        .map(|n| (n, "".to_owned()))
        .collect();
    let outputs = outputs(local_key, &p.entity.ty)?
        .into_iter()
        .map(|n| (n, "".to_owned()))
        .collect();
    Ok(interlayer::Node {
        entity: p.entity,
        res: p.res,
        cloned: p.cloned,
        inputs,
        outputs,
        is_dyn: false,
    })
}

fn translate_conn(
    name: &str,
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
                if node.inputs.contains_key(&p) {
                    rx.push(port);
                    if let Some(p) = node.inputs.get_mut(&p) {
                        *p = name.to_owned();
                    }
                    find = true;
                    break;
                } else if node.outputs.contains_key(&p) {
                    tx.push(port);
                    if let Some(p) = node.outputs.get_mut(&p) {
                        *p = name.to_owned();
                    }
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
        for p in &tx {
            let node = nodes
                .get_mut(&p.node_name)
                .expect("dyn share subgraph is not supported ");
            node.is_dyn = true;
        }
    }

    if dyn_txn > 0 {
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
    p: presentation::Graph,
    shared_nodes: &mut HashMap<String, interlayer::Node>,
) -> Result<interlayer::Graph> {
    let mut nodes = HashMap::new();
    let mut resources = HashMap::new();
    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut connections = HashMap::new();

    for node in p.nodes {
        nodes.insert(node.entity.name.clone(), translate_node(local_key, node)?);
    }
    for res in p.resources {
        resources.insert(res.name.clone(), res);
    }

    for (i, conn) in p.connections.into_iter().enumerate() {
        let name = format!("__{}__", i);
        let conn = translate_conn(&name, conn, &mut nodes, shared_nodes)?;
        connections.insert(name, conn);
    }

    for mut input in p.inputs {
        inputs.push(input.name.clone());
        for p in &mut input.conn.ports {
            let ((n, _), _) = interlayer::Port::parse(p.as_str())?;
            if let Some(node) = nodes.get_mut(n) {
                if node.is_dyn {
                    let conn_name = format!("__{}xDynOutTransform__", input.name);
                    let node_name = conn_name.clone();
                    *p = format!("{}:inp", node_name);
                    let mut tmp_conn = presentation::Connection {
                        cap: input.conn.cap,
                        ports: vec![],
                    };
                    let tmp_node = interlayer::Node {
                        entity: interlayer::Entity {
                            name: node_name,
                            ty: "DynOutTransform".to_owned(),
                            args: toml::value::Table::new(),
                        },
                        res: Default::default(),
                        cloned: Some(1),
                        inputs: HashMap::new(),
                        outputs: HashMap::new(),
                        is_dyn: false,
                    };
                    tmp_conn.ports.push(p.to_owned());
                    nodes.insert(tmp_node.entity.name.clone(), tmp_node);
                    let conn = translate_conn(&conn_name, tmp_conn, &mut nodes, shared_nodes)?;
                    connections.insert(conn_name, conn);
                }
            }
        }
        let conn = translate_conn(&input.name, input.conn, &mut nodes, shared_nodes)?;
        connections.insert(input.name, conn);
    }

    for mut output in p.outputs {
        outputs.push(output.name.clone());
        for p in &mut output.conn.ports {
            let ((n, _), _) = interlayer::Port::parse(p.as_str())?;
            if let Some(node) = nodes.get_mut(n) {
                if node.is_dyn {
                    let conn_name = format!("__{}xDynInTransform__", output.name);
                    let node_name = conn_name.clone();
                    *p = format!("{}:inp", node_name);
                    let mut tmp_conn = presentation::Connection {
                        cap: output.conn.cap,
                        ports: vec![],
                    };
                    let tmp_node = interlayer::Node {
                        entity: interlayer::Entity {
                            name: node_name,
                            ty: "DynInTransform".to_owned(),
                            args: toml::value::Table::new(),
                        },
                        res: Default::default(),
                        cloned: Some(1),
                        inputs: HashMap::new(),
                        outputs: HashMap::new(),
                        is_dyn: false,
                    };
                    tmp_conn.ports.push(p.to_owned());
                    nodes.insert(tmp_node.entity.name.clone(), tmp_node);
                    let conn = translate_conn(&conn_name, tmp_conn, &mut nodes, shared_nodes)?;
                    connections.insert(conn_name, conn);
                }
            }
        }
        let conn = translate_conn(&output.name, output.conn, &mut nodes, shared_nodes)?;
        connections.insert(output.name, conn);
    }

    Ok(interlayer::Graph {
        name: p.name,
        resources,
        nodes,
        inputs,
        outputs,
        connections,
    })
}

pub fn translate_config(local_key: u64, p: presentation::Config) -> Result<interlayer::Config> {
    let mut graphs = vec![];
    let mut nodes = HashMap::new();
    let mut resources = HashMap::new();

    for node in p.nodes {
        nodes.insert(node.entity.name.clone(), translate_node(local_key, node)?);
    }

    for graph in p.graphs {
        graphs.push(translate_graph(local_key, graph, &mut nodes)?);
    }

    for res in p.resources {
        resources.insert(res.name.clone(), res);
    }

    Ok(interlayer::Config {
        graphs,
        resources,
        nodes,
        main: p.main,
    })
}
