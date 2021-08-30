/**
 * \file flow-rs/src/node/noop.rs
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
use toml::value::Table;

#[inputs]
#[outputs(out)]
#[derive(Node, Actor, Default)]
struct NoopProducer {}

impl NoopProducer {
    fn new(_name: String, _args: &Table) -> NoopProducer {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        Ok(())
    }
}

node_register!("NoopProducer", NoopProducer);

#[inputs(inp)]
#[outputs]
#[derive(Node, Actor, Default)]
struct NoopConsumer {}

impl NoopConsumer {
    fn new(_name: String, _args: &Table) -> NoopConsumer {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        self.inp.recv_any().await.ok();
        Ok(())
    }
}

node_register!("NoopConsumer", NoopConsumer);
