/**
 * \file flow-rs/src/debug/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use event_listener::Event;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Feature {
    enable: AtomicBool,
    lock_ops: Event,
}

impl Feature {
    pub const fn new() -> Feature {
        Feature {
            enable: AtomicBool::new(false),
            lock_ops: Event::new(),
        }
    }

    pub fn enable(&self) -> bool {
        self.enable.load(Ordering::Relaxed)
    }

    pub fn disable(&self) {
        self.enable.store(false, Ordering::Relaxed);
    }

    pub fn notify(&self) {
        if self
            .enable
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.lock_ops.notify(usize::MAX);
        }
    }
    #[allow(dead_code)]
    pub async fn wait(&self) {
        let mut listener = None;

        loop {
            if self.enable.load(Ordering::Relaxed) {
                return;
            }

            match listener.take() {
                None => {
                    listener = Some(self.lock_ops.listen());
                }
                Some(l) => {
                    l.await;
                }
            }
        }
    }
}

pub struct FeatureCommand {
    pub start: Box<dyn Fn(usize, serde_json::Value) + Send + Sync>,
    pub stop: Box<dyn Fn(usize) + Send + Sync>,
    pub disable: Box<dyn Fn() + Send + Sync>,
}
crate::collect!(String, FeatureCommand);
