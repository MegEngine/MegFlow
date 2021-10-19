/**
 * \file flow-rs/src/config/insert.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::*;
use std::collections::{BTreeSet, HashMap};

fn global_res_impl<'a>(
    name: String,
    cfg: &'a mut interlayer::Config,
    global: &[String],
    visit: &mut HashMap<String, bool>,
) -> Vec<String> {
    let graph = cfg.graphs.iter().find(|graph| graph.name == name).unwrap();
    if visit[&graph.name] {
        return graph.global_res.clone();
    }
    *visit.get_mut(&graph.name).unwrap() = true;

    let mut capture = BTreeSet::new();
    let mut tmp_mapping = HashMap::new();

    for node in graph.nodes.values() {
        for ty in &node.entity.ty {
            if !visit.contains_key(ty) {
                node.res
                    .iter()
                    .filter(|x| global.contains(x))
                    .for_each(|x| {
                        capture.insert(x.clone());
                    });
            }
        }
    }
    for name in graph
        .nodes
        .values()
        .flat_map(|x| x.entity.ty.iter())
        .filter(|x| visit.contains_key(*x))
        .map(|x| x.to_owned())
        .collect::<Vec<_>>()
    {
        global_res_impl(name.to_owned(), cfg, global, visit)
            .into_iter()
            .for_each(|x| {
                capture.insert(x.clone());
                tmp_mapping
                    .entry(name.to_owned())
                    .or_insert_with(BTreeSet::new)
                    .insert(x);
            });
    }

    let graph = cfg
        .graphs
        .iter_mut()
        .find(|graph| graph.name == name)
        .unwrap();

    for node in graph.nodes.values_mut() {
        for name in &node.entity.ty {
            if visit.contains_key(name) {
                if let Some(graph) = tmp_mapping.get(name) {
                    node.res.append(&mut graph.iter().cloned().collect());
                }
            }
        }
    }

    graph.global_res = capture.into_iter().collect();
    graph.global_res.clone()
}

pub fn global_res(cfg: &mut interlayer::Config) {
    let global_res_names: Vec<_> = cfg.resources.keys().cloned().collect();
    let mut visit: HashMap<_, _> = cfg
        .graphs
        .iter()
        .map(|graph| (graph.name.clone(), false))
        .collect();

    for name in visit.keys().cloned().collect::<Vec<_>>() {
        global_res_impl(name, cfg, global_res_names.as_slice(), &mut visit);
    }
}

pub fn bcast(
    id: &str,
    src: &mut interlayer::Connection,
    connections: &mut HashMap<String, interlayer::Connection>,
    nodes: &mut HashMap<String, interlayer::Node>,
) {
    if src.rx.len() > 1 {
        let node_name = format!("__{}xBcast__", id);
        let bcast_p = |port_name: &str| interlayer::Port {
            node_type: vec!["Bcast".to_owned()],
            node_name: node_name.clone(),
            port_type: interlayer::PortTy::Unit,
            port_name: port_name.to_owned(),
            port_tag: None,
        };

        let rx = std::mem::replace(&mut src.rx, vec![bcast_p("inp")]);

        for (i, p) in rx.into_iter().enumerate() {
            let name = format!("__{}xBcastxOut{}__", id, i);
            connections.insert(
                name,
                interlayer::Connection {
                    cap: src.cap,
                    rx: vec![p],
                    tx: vec![bcast_p("[out]")],
                },
            );
        }

        nodes.insert(
            node_name.clone(),
            interlayer::Node {
                entity: interlayer::Entity {
                    name: node_name,
                    ty: vec!["Bcast".to_owned()],
                    args: Default::default(),
                },
                cloned: Some(1),
                res: vec![],
                is_dyn: false,
                inputs: vec!["inp".to_owned()],
                outputs: vec!["[out]".to_owned()],
                is_shared: false,
            },
        );
    }
}

pub fn dyn_inp_trans(
    src: &mut presentation::NamedConn,
    connections: &mut HashMap<String, interlayer::Connection>,
    nodes: &mut HashMap<String, interlayer::Node>,
    shared_nodes: &mut HashMap<String, interlayer::Node>,
) -> Result<()> {
    for p in &mut src.conn.ports {
        let ((n, _), _) = interlayer::Port::parse(p.as_str())?;
        if let Some(node) = nodes.get_mut(n) {
            if node.is_dyn {
                let conn_name = format!("__{}xDynOutTransform__", src.name);
                let node_name = conn_name.clone();
                *p = format!("{}:inp", node_name);
                let mut tmp_conn = presentation::Connection {
                    cap: src.conn.cap,
                    ports: vec![],
                };
                let tmp_node = interlayer::Node {
                    entity: interlayer::Entity {
                        name: node_name,
                        ty: vec!["DynOutTransform".to_owned()],
                        args: toml::value::Table::new(),
                    },
                    res: Default::default(),
                    cloned: Some(1),
                    inputs: vec!["inp".to_owned()],
                    outputs: vec!["dyn@out".to_owned()],
                    is_dyn: false,
                    is_shared: false,
                };
                tmp_conn.ports.push(p.to_owned());
                nodes.insert(tmp_node.entity.name.clone(), tmp_node);
                let conn = translate_conn(tmp_conn, nodes, shared_nodes)?;
                connections.insert(conn_name, conn);
            }
        }
    }
    Ok(())
}

pub fn dyn_out_trans(
    src: &mut presentation::NamedConn,
    connections: &mut HashMap<String, interlayer::Connection>,
    nodes: &mut HashMap<String, interlayer::Node>,
    shared_nodes: &mut HashMap<String, interlayer::Node>,
) -> Result<()> {
    for p in &mut src.conn.ports {
        let ((n, _), _) = interlayer::Port::parse(p.as_str())?;
        if let Some(node) = nodes.get_mut(n) {
            if node.is_dyn {
                let conn_name = format!("__{}xDynInTransform__", src.name);
                let node_name = conn_name.clone();
                *p = format!("{}:inp", node_name);
                let mut tmp_conn = presentation::Connection {
                    cap: src.conn.cap,
                    ports: vec![],
                };
                let tmp_node = interlayer::Node {
                    entity: interlayer::Entity {
                        name: node_name,
                        ty: vec!["DynInTransform".to_owned()],
                        args: toml::value::Table::new(),
                    },
                    res: Default::default(),
                    cloned: Some(1),
                    inputs: vec!["dyn@inp".to_owned()],
                    outputs: vec!["out".to_owned()],
                    is_dyn: false,
                    is_shared: false,
                };
                tmp_conn.ports.push(p.to_owned());
                nodes.insert(tmp_node.entity.name.clone(), tmp_node);
                let conn = translate_conn(tmp_conn, nodes, shared_nodes)?;
                connections.insert(conn_name, conn);
            }
        }
    }

    Ok(())
}
