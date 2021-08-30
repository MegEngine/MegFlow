/**
 * \file flow-derive/examples/node_derive.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use anyhow::Result;
use flow_rs::prelude::*;

#[allow(dead_code)]
#[inputs(first, second:dyn, third:[])]
#[outputs(front, last:dyn)]
#[derive(Node, Actor, Default)]
struct SubNode {}

impl SubNode {
    fn new(_: String, _args: &toml::value::Table) -> Self {
        SubNode {
            ..Default::default()
        }
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}

    async fn exec(&mut self, _: &Context) -> Result<()> {
        let _: Envelope<f32> = self.first.recv::<f32>().await.unwrap();
        Ok(())
    }
}

node_register!("sub", SubNode);

fn main() {}
