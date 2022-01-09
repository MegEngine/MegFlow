/**
 * \file flow-rs/src/node/transform.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::rt;
use anyhow::Result;
use flow_rs::prelude::*;
use toml::value::Table;

#[inputs(inp)]
#[outputs(out)]
#[derive(Node, Actor, Default)]
struct Transform {}

impl Transform {
    fn new(_name: String, _args: &Table) -> Transform {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        if let Ok(msg) = self.inp.recv_any().await {
            self.out.send_any(msg).await.ok();
        }
        Ok(())
    }
}

node_register!("Transform", Transform);

#[inputs(inp)]
#[outputs(out:dyn)]
#[derive(Node, Default)]
struct DynOutTransform {}

impl DynOutTransform {
    fn new(_name: String, _args: &Table) -> DynOutTransform {
        Default::default()
    }
}

impl Actor for DynOutTransform {
    fn start(
        mut self: Box<Self>,
        _: Context,
        _: ResourceCollection,
    ) -> rt::task::JoinHandle<Result<()>> {
        let (s, r) = rt::channel::unbounded();
        rt::task::spawn(async move {
            while let Ok((_, out)) = self.out.fetch().await {
                let inp = self.inp.clone();
                s.send(rt::task::spawn(async move {
                    let mut empty_n = 0;
                    loop {
                        while let Ok(msg) = inp.recv_any().await {
                            out.send_any(msg).await.ok();
                        }
                        if inp.is_closed() {
                            break;
                        }
                        let n = inp.empty_n();
                        for _ in empty_n..n {
                            out.send_any(DummyEnvelope {}.seal()).await.ok();
                        }
                        empty_n = n;
                    }
                }))
                .await
                .ok();
            }
        });
        rt::task::spawn(async move {
            while let Ok(handle) = r.recv().await {
                handle.await;
            }
            Ok(())
        })
    }
}

node_register!("DynOutTransform", DynOutTransform);

#[inputs(inp: dyn)]
#[outputs(out)]
#[derive(Node, Default)]
struct DynInTransform {}

impl DynInTransform {
    fn new(_name: String, _args: &Table) -> DynInTransform {
        Default::default()
    }
}

impl Actor for DynInTransform {
    fn start(
        self: Box<Self>,
        _: Context,
        _: ResourceCollection,
    ) -> rt::task::JoinHandle<Result<()>> {
        let (s, r) = rt::channel::unbounded();
        rt::task::spawn(async move {
            while let Ok((_, inp)) = self.inp.fetch().await {
                let out = self.out.clone();
                s.send(rt::task::spawn(async move {
                    while !inp.is_closed() {
                        while let Ok(msg) = inp.recv_any().await {
                            out.send_any(msg).await.ok();
                        }
                        assert!(
                            inp.empty_n() == 0,
                            "DynInTransform is not supported in a shared subgraph"
                        );
                    }
                }))
                .await
                .ok();
            }
        });
        rt::task::spawn(async move {
            while let Ok(handle) = r.recv().await {
                handle.await;
            }
            Ok(())
        })
    }
}

node_register!("DynInTransform", DynInTransform);
