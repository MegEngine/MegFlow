/**
 * \file flow-rs/src/graph/mod.rs
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
use std::sync::Arc;

pub struct ContextInner {
    pub name: String,
    pub ty: String,
    pub local_key: u64,
    lock_ops: Event,
    is_closed: AtomicBool,
}

pub type Context = Arc<ContextInner>;

impl ContextInner {
    pub fn close(&self) {
        self.is_closed.store(true, Ordering::Release);
        self.lock_ops.notify(usize::MAX);
    }
    pub fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Relaxed)
    }
    pub async fn wait(&self) {
        let mut listener = None;

        loop {
            if self.is_closed.load(Ordering::Relaxed) {
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

pub(crate) fn context(name: String, ty: String, local_key: u64) -> Context {
    Arc::new(ContextInner {
        name,
        ty,
        local_key,
        lock_ops: Event::new(),
        is_closed: AtomicBool::new(false),
    })
}
