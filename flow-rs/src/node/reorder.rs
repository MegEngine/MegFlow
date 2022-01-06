/**
 * \file flow-rs/src/node/reorder.rs
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
use std::collections::BTreeMap;
use toml::value::Table;

#[inputs(inp)]
#[outputs(out)]
#[derive(Node, Actor, Default)]
struct Reorder {
    cache: BTreeMap<u64, SealedEnvelope>,
    seq_id: u64,
}

impl Reorder {
    fn new(_: String, _: &Table) -> Reorder {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}

    async fn exec(&mut self, _: &Context) -> Result<()> {
        if let Ok(msg) = self.inp.recv_any().await {
            let id = msg
                .info()
                .partial_id
                .expect("partial_id required by reorder");
            assert!(id >= self.seq_id);
            if id == self.seq_id {
                self.seq_id += 1;
                self.out.send_any(msg).await.ok();
            } else {
                self.cache.insert(id, msg);
            }

            let mut stop = self.seq_id;
            for &id in self.cache.keys() {
                assert!(id >= self.seq_id);
                if id == self.seq_id {
                    self.seq_id += 1;
                } else {
                    stop = id;
                    break;
                }
            }
            if stop != self.seq_id {
                let rest = if stop > self.seq_id {
                    let rest = self.cache.split_off(&stop);
                    std::mem::replace(&mut self.cache, rest)
                } else {
                    std::mem::take(&mut self.cache)
                };

                for (_, msg) in rest {
                    self.out.send_any(msg).await.ok();
                }
            }
        } else {
            assert!(self.cache.is_empty());
        }
        Ok(())
    }
}

node_register!("Reorder", Reorder);

#[cfg(test)]
mod test {
    use crate::envelope::Envelope;
    use crate::sandbox::Sandbox;
    use futures_util::join;
    use rand::prelude::*;
    use rand::seq::SliceRandom;
    #[flow_rs::rt::test]
    async fn test_reorder() {
        let reorder = Sandbox::pure("Reorder").unwrap();
        let input = reorder.input("inp").unwrap();
        let output = reorder.output("out").unwrap();
        let handle = reorder.start();
        let send_fut = async move {
            let mut rng: StdRng = SeedableRng::from_entropy();
            let mut list: Vec<u64> = (0..10).collect();
            list[..].shuffle(&mut rng);
            for i in list {
                let mut envelope = Envelope::new(0usize).seal();
                envelope.info_mut().partial_id = Some(i);
                input.send_any(envelope).await.ok();
            }
            input.close();
        };

        let recv_fut = async move {
            let mut seq_id = 0;
            while let Ok(msg) = output.recv_any().await {
                assert_eq!(msg.info().partial_id, Some(seq_id));
                seq_id += 1;
            }
        };

        join!(send_fut, recv_fut);

        handle.await.unwrap();
    }
}
