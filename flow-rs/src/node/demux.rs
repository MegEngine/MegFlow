/**
 * \file flow-rs/src/node/demux.rs
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
use rt::task::JoinHandle;
use std::collections::HashMap;
use toml::value::Table;

#[inputs(inp)]
#[outputs(out:dyn)]
#[derive(Node, Actor, Default)]
struct DynDemux {
    tasks: HashMap<u64, JoinHandle<Result<()>>>,
    resources: Option<ResourceCollection>,
}

impl DynDemux {
    fn new(_: String, _: &Table) -> DynDemux {
        Default::default()
    }

    async fn initialize(&mut self, resources: ResourceCollection) {
        self.resources = Some(resources);
    }
    async fn finalize(&mut self) {
        for (_, task) in std::mem::take(&mut self.tasks) {
            task.await.ok();
        }
    }

    async fn exec(&mut self, _: &Context) -> Result<()> {
        if let Ok(msg) = self.inp.recv_any().await {
            let id = msg
                .info()
                .to_addr
                .expect("the envelope has no destination address");
            let tag = msg.info().tag.as_ref();
            if msg.is_none() {
                self.out.fetch_with_cache().await.remove(&id);
                if let Some(task) = self.tasks.remove(&id) {
                    task.await.ok();
                }
            } else {
                if !self.out.cache().contains_key(&id) {
                    if let Some(tag) = tag {
                        self.tasks.insert(
                            id,
                            self.out
                                .create_spec(
                                    id,
                                    tag,
                                    self.resources.clone().unwrap(),
                                    Default::default(),
                                )
                                .await
                                .expect("create subgraph fault"),
                        );
                    } else {
                        self.tasks.insert(
                            id,
                            self.out
                                .create(id, self.resources.clone().unwrap(), Default::default())
                                .await
                                .expect("create subgraph fault"),
                        );
                    }
                }
                let out = self.out.fetch_with_cache().await.get(&id).unwrap();
                out.send_any(msg).await.ok();
            }
        }
        Ok(())
    }
}

node_register!("DynDemux", DynDemux);

#[inputs(inp)]
#[outputs(out:{})]
#[derive(Node, Actor, Default)]
struct Demux {}

impl Demux {
    fn new(_: String, _: &Table) -> Demux {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}

    async fn exec(&mut self, _: &Context) -> Result<()> {
        if let Ok(msg) = self.inp.recv_any().await {
            let id = msg
                .info()
                .to_addr
                .expect("the envelope has no destination address");
            if let Some(out) = self.out.get(&id) {
                out.send_any(msg).await.ok();
            }
        }
        Ok(())
    }
}

node_register!("Demux", Demux);

#[cfg(test)]
mod test {
    use crate::envelope::Envelope;
    use crate::sandbox::Sandbox;
    #[flow_rs::rt::test]
    async fn test_demux() {
        let demux = Sandbox::pure("Demux").unwrap();
        let input = demux.input("inp").unwrap();
        let output = demux.output("out").unwrap();
        let handle = demux.start();
        let mut envelope = Envelope::new(0usize).seal();
        envelope.info_mut().to_addr = Some(0);
        input.send_any(envelope).await.ok();
        let envelope = output.recv_any().await;
        assert!(envelope.is_ok());
        let envelope = envelope.unwrap();
        assert_eq!(envelope.info().to_addr, Some(0));
        input.close();
        handle.await.unwrap();
    }
}
