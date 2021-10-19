/**
 * \file flow-rs/src/channel/storage.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::{inner, ChannelBase, Receiver, Sender, SenderRecord};
use crate::envelope::SealedEnvelope;
use crate::rt::sync::Mutex;
use event_listener::Event;
use std::collections::HashMap;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChannelStorage {
    storage: Arc<inner::Channel<SealedEnvelope>>,
    receiver_epoch: Arc<AtomicUsize>,
    sender_epoch: Arc<AtomicUsize>,
    sender_record: SenderRecord,
    rx_counter: Arc<AtomicUsize>,
    tx_counter: Arc<AtomicUsize>,
    rx_close_ops: Arc<Event>,
    tx_close_ops: Arc<Event>,
}

impl ChannelBase for ChannelStorage {
    fn is_closed(&self) -> bool {
        self.storage.queue.is_closed()
    }
    fn is_none(&self) -> bool {
        false
    }
}

impl ChannelStorage {
    pub fn bound(cap: usize) -> ChannelStorage {
        ChannelStorage {
            storage: inner::bounded(cap),
            receiver_epoch: Arc::new(AtomicUsize::new(0)),
            sender_epoch: Arc::new(AtomicUsize::new(0)),
            sender_record: Arc::new(Mutex::new(HashMap::new())),
            rx_counter: Arc::new(AtomicUsize::new(0)),
            tx_counter: Arc::new(AtomicUsize::new(0)),
            rx_close_ops: Arc::new(Event::new()),
            tx_close_ops: Arc::new(Event::new()),
        }
    }
    pub fn unbound() -> ChannelStorage {
        ChannelStorage {
            storage: inner::unbounded(),
            receiver_epoch: Arc::new(AtomicUsize::new(0)),
            sender_epoch: Arc::new(AtomicUsize::new(0)),
            sender_record: Arc::new(Mutex::new(HashMap::new())),
            tx_close_ops: Arc::new(Event::new()),
            rx_close_ops: Arc::new(Event::new()),
            rx_counter: Arc::new(AtomicUsize::new(0)),
            tx_counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn sender(&self) -> Sender {
        let count = self.storage.sender_count.fetch_add(1, Ordering::Relaxed);

        if count > usize::MAX / 2 {
            process::abort();
        }
        Sender::new(
            inner::Sender {
                channel: self.storage.clone(),
                close_ops: self.tx_close_ops.clone(),
            },
            self.sender_epoch.clone(),
            self.sender_record.clone(),
            self.tx_counter.clone(),
        )
    }

    pub fn receiver(&self) -> Receiver {
        let count = self.storage.receiver_count.fetch_add(1, Ordering::Relaxed);

        if count > usize::MAX / 2 {
            process::abort();
        }

        Receiver::new(
            inner::Receiver {
                channel: self.storage.clone(),
                listener: None,
                close_ops: self.rx_close_ops.clone(),
            },
            self.receiver_epoch.clone(),
            self.rx_counter.clone(),
        )
    }

    pub fn len(&self) -> usize {
        self.storage.queue.len()
    }

    pub fn is_almost_full(&self) -> bool {
        match self.storage.queue.capacity() {
            Some(capacity) => (0.9 * capacity as f64) <= self.len() as f64,
            None => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn close(&self) {
        self.storage.close();
    }

    pub fn sender_count(&self) -> usize {
        self.storage.sender_count.load(Ordering::Relaxed)
    }

    pub fn receiver_count(&self) -> usize {
        self.storage.receiver_count.load(Ordering::Relaxed)
    }

    pub fn swap_tx_counter(&self) -> usize {
        self.tx_counter.swap(0, Ordering::Relaxed)
    }

    pub fn swap_rx_counter(&self) -> usize {
        self.rx_counter.swap(0, Ordering::Relaxed)
    }

    pub async fn wait_rx_closed(&self) {
        let mut listener = None;
        loop {
            if self.is_closed() {
                return;
            }
            match listener.take() {
                None => {
                    listener = Some(self.rx_close_ops.listen());
                }
                Some(l) => {
                    l.await;
                }
            }
        }
    }

    pub async fn wait_tx_closed(&self) {
        let mut listener = None;
        loop {
            if self.is_closed() {
                return;
            }
            match listener.take() {
                None => {
                    listener = Some(self.tx_close_ops.listen());
                }
                Some(l) => {
                    l.await;
                }
            }
        }
    }
}
