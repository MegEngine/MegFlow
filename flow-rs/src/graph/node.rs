/**
 * \file flow-rs/src/graph/node.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::config::interlayer as config;
use crate::config::table::merge_table;
use crate::node::Actor;
use anyhow::Result;
use toml::value::Table;

pub struct AnyNode {
    nodes: Vec<Box<dyn Actor>>,
    #[allow(dead_code)]
    info: config::Node,
}

impl AnyNode {
    pub fn new(local_key: u64, mut info: config::Node, extra_args: Table) -> Result<AnyNode> {
        info.entity.args = merge_table(extra_args, info.entity.args);
        Ok(AnyNode {
            nodes: crate::node::load_static(local_key, &info)?,
            info,
        })
    }

    #[allow(dead_code)]
    pub fn first(&self) -> &dyn Actor {
        self.nodes.first().map(|n| n.as_ref()).unwrap()
    }

    #[allow(dead_code)]
    pub fn get(&self) -> &Vec<Box<dyn Actor>> {
        &self.nodes
    }

    pub fn get_mut(&mut self) -> &mut Vec<Box<dyn Actor>> {
        &mut self.nodes
    }

    pub fn get_into(&mut self) -> Vec<Box<dyn Actor>> {
        std::mem::take(&mut self.nodes)
    }

    pub fn info(&self) -> &config::Node {
        &self.info
    }

    pub fn info_mut(&mut self) -> &mut config::Node {
        &mut self.info
    }
}
