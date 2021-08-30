/**
 * \file flow-rs/src/config/interlayer.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum PortTy {
    Unit,
    List,
    Dyn,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub node_type: String,
    pub node_name: String,
    pub port_name: String,
    pub port_type: PortTy,
    pub port_tag: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub cap: usize,
    pub tx: Vec<Port>,
    pub rx: Vec<Port>,
}

pub type Entity = super::presentation::Entity;

#[derive(Clone, Debug)]
pub struct Node {
    pub entity: Entity,
    pub res: Vec<String>,
    pub cloned: Option<usize>,
    pub inputs: HashMap<String, String>,
    pub outputs: HashMap<String, String>,
    pub is_dyn: bool,
}

#[derive(Clone, Debug)]
pub struct Graph {
    pub name: String,
    pub resources: HashMap<String, Entity>,
    pub nodes: HashMap<String, Node>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub connections: HashMap<String, Connection>,
}

#[derive(Debug)]
pub struct Config {
    pub resources: HashMap<String, Entity>,
    pub nodes: HashMap<String, Node>,
    pub graphs: Vec<Graph>,
    pub main: String,
}

impl Port {
    pub fn parse(name: &str) -> Result<((&str, &str), Option<u64>)> {
        let mut iter = name.splitn(3, ':');
        Ok((
            iter.next()
                .zip(iter.next())
                .ok_or_else(|| anyhow!("unexpected port format {}", name))?,
            iter.next()
                .map(|s| s.parse())
                .map_or(Ok(None), |r| r.map(Some))?,
        ))
    }

    pub fn is_dyn(&self) -> bool {
        matches!(self.port_type, PortTy::Dyn)
    }
}
