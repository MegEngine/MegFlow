/**
 * \file flow-rs/src/node/port.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::broker::BrokerClient;
use crate::channel::{ChannelStorage, Receiver, Sender};
use crate::future::select_ok;
use crate::graph::GraphSlice;
use crate::prelude::ChannelBase;
use crate::registry::Collect;
use crate::resource::ResourceCollection;
use crate::rt::task::JoinHandle;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::marker::PhantomData;

use super::Node;

#[derive(Clone)]
struct DynConns {
    name: u64,
    inputs: HashMap<String, Sender>,
    outputs: HashMap<String, Receiver>,
}

#[derive(Default)]
pub struct DynPorts<V> {
    local_key: u64,
    target: String,
    cap: usize,
    brokers: HashMap<String, BrokerClient>,
    cache: HashMap<u64, V>,
    _v_holder: PhantomData<V>,
}

impl<V> DynPorts<V> {
    pub fn new(
        local_key: u64,
        target: String,
        cap: usize,
        brokers: Vec<BrokerClient>,
    ) -> DynPorts<V> {
        DynPorts {
            local_key,
            target,
            cap,
            brokers: brokers
                .into_iter()
                .map(|x| (x.topic().to_owned(), x))
                .collect(),
            cache: HashMap::new(),
            _v_holder: Default::default(),
        }
    }

    pub async fn create(
        &mut self,
        key: u64,
        resource: ResourceCollection,
    ) -> Result<JoinHandle<()>> {
        self.create_spec(
            key,
            self.brokers
                .keys()
                .next()
                .expect("not init")
                .clone()
                .as_str(),
            resource,
        )
        .await
    }

    pub async fn create_spec(
        &mut self,
        key: u64,
        which: &str,
        resource: ResourceCollection,
    ) -> Result<JoinHandle<()>> {
        let broker = self
            .brokers
            .get(which)
            .ok_or_else(|| anyhow!("{} not found", which))?;
        if let Some(slice) = GraphSlice::registry_local()
            .get(self.local_key)
            .get(broker.topic())
        {
            let mut g = (slice.cons)(format!("{}_instance", broker.topic()))?;

            let mut inputs = HashMap::new();
            let mut outputs = HashMap::new();

            for input in &slice.info.inputs {
                let channel = ChannelStorage::bound(self.cap);
                g.set_port(input.as_str(), None, &channel);
                inputs.insert(input.clone(), channel.sender());
            }

            for output in &slice.info.outputs {
                let channel = ChannelStorage::bound(self.cap);
                g.set_port(output.as_str(), None, &channel);
                outputs.insert(output.clone(), channel.receiver());
            }

            let handle = g.start(Some(resource));

            broker
                .publish(DynConns {
                    name: key,
                    inputs,
                    outputs,
                })
                .await;
            Ok(handle)
        } else {
            Err(anyhow!("{} not found", broker.topic()))
        }
    }
}

impl DynPorts<Receiver> {
    pub fn try_fetch(&self) -> Option<(u64, Receiver)> {
        for broker in self.brokers.values() {
            if let Some(mut conns) = broker.try_fetch::<DynConns>() {
                let conn = conns.outputs.remove(&self.target).unwrap_or_else(|| {
                    panic!("port {} not found in graph {}", self.target, broker.topic())
                });
                return Some((conns.name, conn));
            }
        }
        None
    }

    pub async fn fetch(&self) -> Result<(u64, Receiver)> {
        let fut: Vec<_> = self
            .brokers
            .values()
            .map(|x| x.fetch::<DynConns>())
            .collect();
        if let Ok(mut conns) = select_ok(fut).await {
            let conn = conns.outputs.remove(&self.target).unwrap_or_else(|| {
                panic!(
                    "port {} not found in graph {:?}",
                    self.target,
                    self.brokers.keys()
                )
            });
            return Ok((conns.name, conn));
        }
        Err(anyhow!("broker is closed"))
    }

    pub async fn fetch_with_cache(&mut self) -> &mut HashMap<u64, Receiver> {
        if self.cache.is_empty() {
            if let Ok((k, v)) = self.fetch().await {
                self.cache.insert(k, v);
            }
        } else if let Some((k, v)) = self.try_fetch() {
            self.cache.insert(k, v);
        }
        &mut self.cache
    }

    pub fn cache(&self) -> &HashMap<u64, Receiver> {
        &self.cache
    }

    pub fn is_closed(&self) -> bool {
        let mut is_closed = self
            .brokers
            .values()
            .fold(false, |is_closed, broker| is_closed && broker.is_closed());
        for chan in self.cache.values() {
            is_closed = is_closed && chan.is_closed();
        }
        is_closed
    }
}

impl DynPorts<Sender> {
    pub fn try_fetch(&mut self) -> Option<(u64, Sender)> {
        for broker in self.brokers.values() {
            if let Some(mut conns) = broker.try_fetch::<DynConns>() {
                let conn = conns.inputs.remove(&self.target).unwrap_or_else(|| {
                    panic!("port {} not found in graph {}", self.target, broker.topic())
                });
                return Some((conns.name, conn));
            }
        }
        None
    }

    pub async fn fetch(&mut self) -> Result<(u64, Sender)> {
        let fut: Vec<_> = self
            .brokers
            .values()
            .map(|x| x.fetch::<DynConns>())
            .collect();
        if let Ok(mut conns) = select_ok(fut).await {
            let conn = conns.inputs.remove(&self.target).unwrap_or_else(|| {
                panic!(
                    "port {} not found in graph {:?}",
                    self.target,
                    self.brokers.keys()
                )
            });
            return Ok((conns.name, conn));
        }
        Err(anyhow!("broker is closed"))
    }

    pub async fn fetch_with_cache(&mut self) -> &mut HashMap<u64, Sender> {
        if self.cache.is_empty() {
            if let Ok((k, v)) = self.fetch().await {
                self.cache.insert(k, v);
            }
        } else if let Some((k, v)) = self.try_fetch() {
            self.cache.insert(k, v);
        }
        &mut self.cache
    }

    pub fn cache(&self) -> &HashMap<u64, Sender> {
        &self.cache
    }

    pub fn close(&self) {
        for broker in self.brokers.values() {
            broker.close();
        }
        for chan in self.cache.values() {
            chan.close();
        }
    }
}
