/**
 * \file flow-rs/src/config/presentation.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use flow_rs::prelude::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use toml::value::Table;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub cap: usize,
    pub ports: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NamedConn {
    pub name: String,
    #[serde(flatten)]
    pub conn: Connection,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub ty: String,
    #[serde(default, flatten)]
    pub args: Table,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    #[serde(flatten)]
    pub entity: Entity,
    #[serde(default)]
    pub res: Vec<String>,
    pub cloned: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Parser)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub name: String,
    #[serde(default)]
    pub resources: Vec<Entity>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub inputs: Vec<NamedConn>,
    #[serde(default)]
    pub outputs: Vec<NamedConn>,
    #[serde(default)]
    pub connections: Vec<Connection>,
}

#[derive(Serialize, Deserialize, Debug, Parser)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub include: Vec<PathBuf>,
    #[serde(default)]
    pub resources: Vec<Entity>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    pub graphs: Vec<Graph>,
    pub main: String,
}
