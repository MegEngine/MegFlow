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
    broker: Option<BrokerClient>,
    cache: HashMap<u64, V>,
    _v_holder: PhantomData<V>,
}

impl<V> DynPorts<V> {
    pub fn new(local_key: u64, target: String, cap: usize, broker: BrokerClient) -> DynPorts<V> {
        DynPorts {
            local_key,
            target,
            cap,
            broker: Some(broker),
            cache: HashMap::new(),
            _v_holder: Default::default(),
        }
    }

    pub async fn create(
        &mut self,
        key: u64,
        resource: ResourceCollection,
    ) -> Result<JoinHandle<()>> {
        let broker = self.broker.as_ref().expect("not init");
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
        if let Some(broker) = self.broker.as_ref() {
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
        if let Some(broker) = self.broker.as_ref() {
            if let Ok(mut conns) = broker.fetch::<DynConns>().await {
                let conn = conns.outputs.remove(&self.target).unwrap_or_else(|| {
                    panic!("port {} not found in graph {}", self.target, broker.topic())
                });
                return Ok((conns.name, conn));
            }
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
            .broker
            .as_ref()
            .map(|broker| broker.is_closed())
            .unwrap_or(false);
        for chan in self.cache.values() {
            is_closed = is_closed && chan.is_closed();
        }
        is_closed
    }
}

impl DynPorts<Sender> {
    pub fn try_fetch(&mut self) -> Option<(u64, Sender)> {
        if let Some(broker) = self.broker.as_ref() {
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
        if let Some(broker) = self.broker.as_ref() {
            if let Ok(mut conns) = broker.fetch::<DynConns>().await {
                let conn = conns.inputs.remove(&self.target).unwrap_or_else(|| {
                    panic!("port {} not found in graph {}", self.target, broker.topic())
                });
                return Ok((conns.name, conn));
            }
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
        if let Some(broker) = &self.broker {
            broker.close();
        }
        for chan in self.cache.values() {
            chan.close();
        }
    }
}
