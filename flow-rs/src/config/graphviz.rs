/**
 * \file flow-rs/src/config/graphviz.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::interlayer::*;
use anyhow::Result;
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub fn dump(config: &Config) -> Result<()> {
    if let Ok(path) = std::env::var("MEGFLOW_DUMP") {
        let dot = dump_dot(config);
        let path = PathBuf::from(path);
        let mut f = File::create(&path)?;
        write!(f, "{}", dot)?;
        let output = Command::new("dot").arg("-Tpng").arg(&path).output()?;
        std::fs::write(&path, output.stdout)?;
    }
    Ok(())
}

fn dump_dot(config: &Config) -> String {
    let mut buf = vec![];
    let mut rename = HashMap::new();
    let mut flatten_ports = HashMap::new();
    let mut dc = 0;
    let mut nc = 0;
    let mut mapping = |name| {
        let id = rename.entry(name).or_insert(nc);
        if id == &nc {
            nc += 1;
        }
        *id
    };
    let is_graph = |name: &str| config.graphs.iter().any(|x| x.name == name);
    // flatten subgraph i/o ports
    for graph in &config.graphs {
        for pname in graph.inputs.iter().chain(graph.outputs.iter()) {
            let conn = graph.connections.get(pname).unwrap();
            flatten_ports.insert(
                format!("{}:{}", graph.name, pname),
                conn.tx
                    .iter()
                    .chain(conn.rx.iter())
                    .map(|x| {
                        if config.nodes.contains_key(&x.node_name) {
                            ("global".to_owned(), x.node_name.clone())
                        } else {
                            (graph.name.clone(), x.node_name.clone())
                        }
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
    buf.push(format!("digraph {} {{", config.main));

    // write subgraph into buf
    for (i, graph) in config.graphs.iter().enumerate() {
        let is_subgraph = graph.name != config.main;
        if is_subgraph {
            buf.push(format!("subgraph cluster_{} {{", i));
            buf.push("style=filled;color=lightgrey;node [style=filled,color=white];".to_string());
        }
        // write nodes into buf
        for (name, _) in graph
            .nodes
            .iter()
            .filter(|(_, config)| !config.entity.ty.iter().any(|x| is_graph(x)))
        {
            let id = mapping((graph.name.clone(), name));
            buf.push(format!("n{} [label=\"{}\"]", id, name));
        }
        // write connections into buf
        for (name, conn) in &graph.connections {
            macro_rules! extract {
                ($x: ident, $x_len: ident) => {
                    let mut $x = vec![];
                    for p in &conn.$x {
                        for ty in &p.node_type {
                            let pname = format!("{}:{}", ty, p.port_name);
                            if flatten_ports.contains_key(&pname) {
                                let flatten_ports = flatten_ports.get(&pname).unwrap();
                                let mut flatten_ports =
                                    flatten_ports.iter().map(|p| mapping((p.0.clone(), &p.1))).collect();
                                $x.append(&mut flatten_ports);
                            } else if config.nodes.contains_key(&p.node_name) {
                                $x.push(mapping(("global".to_owned(), &p.node_name)));
                            } else {
                                $x.push(mapping((graph.name.clone(), &p.node_name)));
                            }
                        }

                    }
                    if $x.is_empty() {
                        if is_subgraph {
                            continue;
                        } else {
                            let id = mapping((graph.name.clone(), name));
                            buf.push(format!(
                                "n{} [label=\"{}\", shape=circle,fillcolor=black, style=filled, fontcolor=white]",
                                id, name
                            ));
                            $x.push(id);
                        }
                    }
                    let ($x_len, $x) = (
                        $x.len(),
                        $x.iter()
                            .map(|x| format!("n{}", x))
                            .collect::<BTreeSet<_>>()
                            .into_iter()
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                };
            }

            extract!(tx, tx_len);
            extract!(rx, rx_len);

            if tx_len == 1 && rx_len == 1 {
                buf.push(format!("{} -> {}", tx, rx));
            } else {
                buf.push(format!("dc{} [shape=point,width=0.01,height=0.01]", dc));
                buf.push(format!("{} -> dc{} [dir=none]", tx, dc));
                buf.push(format!("dc{} -> {}", dc, rx));
                dc += 1;
            }
        }

        if is_subgraph {
            buf.push(format!("label = \"{}\"", graph.name));
            buf.push("}".to_string());
        }
    }
    // write nodes in global into buf
    for (name, _) in config
        .nodes
        .iter()
        .filter(|(_, config)| !config.entity.ty.iter().any(|x| is_graph(x)))
    {
        let id = mapping(("global".to_owned(), name));
        buf.push(format!("n{} [label=\"{}\"]", id, name));
    }
    buf.push("}".to_string());
    buf.join("\n")
}
