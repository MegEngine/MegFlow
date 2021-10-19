/**
 * \file flow-rs/src/node/bcast.rs
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

#[inputs(inp)]
#[outputs(out:[])]
#[derive(Node, Actor, Default)]
struct Bcast {}

impl Bcast {
    fn new(_: String, _: &Table) -> Bcast {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}

    async fn exec(&mut self, _: &Context) -> Result<()> {
        if let Ok(msg) = self.inp.recv_any().await {
            for out in &self.out[0..self.out.len() - 1] {
                let msg_cloned = msg.clone();
                out.send_any(msg_cloned).await.ok();
            }
            if let Some(out) = self.out.last() {
                out.send_any(msg).await.ok();
            }
        }
        Ok(())
    }
}

node_register!("Bcast", Bcast);

#[cfg(test)]
mod test {
    use crate::envelope::Envelope;
    use crate::sandbox::Sandbox;
    #[flow_rs::rt::test]
    async fn test_bcast() {
        let bcast = Sandbox::pure("Bcast").unwrap();
        let input = bcast.input("inp").unwrap();
        let output = bcast.output("out").unwrap();
        let handle = bcast.start();
        input.send(Envelope::new(0usize)).await.ok();
        let envelope = output.recv::<usize>().await;
        assert!(envelope.is_ok());
        let envelope = envelope.unwrap();
        assert_eq!(envelope.get_ref(), &0);
        assert!(output.is_empty());
        input.close();
        handle.await;
    }
}
