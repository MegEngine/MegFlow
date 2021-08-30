/**
 * \file flow-rs/src/graph/channel.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::channel::ChannelStorage;
use crate::config::interlayer as config;
use anyhow::Result;

#[derive(Clone)]
pub struct AnyChannel {
    storage: Option<ChannelStorage>,
    info: config::Connection,
}

impl AnyChannel {
    pub fn new(cfg: &config::Connection) -> Result<AnyChannel> {
        Ok(AnyChannel {
            storage: None,
            info: cfg.clone(),
        })
    }

    pub fn set(&mut self, storage: ChannelStorage) {
        self.storage = Some(storage)
    }

    pub fn make(&self) -> ChannelStorage {
        ChannelStorage::bound(self.info.cap)
    }

    pub fn get(&self) -> &ChannelStorage {
        self.storage.as_ref().unwrap()
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self) -> &mut ChannelStorage {
        self.storage.as_mut().unwrap()
    }

    pub fn info(&self) -> &config::Connection {
        &self.info
    }
}
