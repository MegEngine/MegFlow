/**
 * \file flow-rs/src/channel/sender.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::rt::sync::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::inner::Sender as SendImpl;
use super::SendError;
use crate::envelope::{DummyEnvelope, Envelope, SealedEnvelope};

use super::ChannelBase;

pub type SenderRecord = Arc<Mutex<HashMap<usize, usize>>>;

#[derive(Default)]
pub struct Sender {
    imp: Option<SendImpl<SealedEnvelope>>,
    epoch: AtomicUsize,
    max_epoch: Arc<AtomicUsize>,
    record: SenderRecord,
    counter: Arc<AtomicUsize>,
}

impl Clone for Sender {
    fn clone(&self) -> Self {
        let epoch = self.max_epoch.load(Ordering::Relaxed);

        Sender {
            imp: self.imp.clone(),
            epoch: AtomicUsize::new(epoch),
            max_epoch: self.max_epoch.clone(),
            record: self.record.clone(),
            counter: self.counter.clone(),
        }
    }
}

impl ChannelBase for Sender {
    fn is_closed(&self) -> bool {
        if let Some(imp) = self.imp.as_ref() {
            imp.is_closed()
        } else {
            true
        }
    }
    fn is_none(&self) -> bool {
        self.imp.is_none()
    }
}

impl Sender {
    pub fn new(
        imp: SendImpl<SealedEnvelope>,
        max_epoch: Arc<AtomicUsize>,
        record: SenderRecord,
        counter: Arc<AtomicUsize>,
    ) -> Self {
        let epoch = max_epoch.load(Ordering::Relaxed);
        Sender {
            imp: Some(imp),
            epoch: AtomicUsize::new(epoch),
            max_epoch,
            record,
            counter,
        }
    }

    #[doc(hidden)]
    pub fn empty_n(&self) -> usize {
        self.epoch.load(Ordering::Relaxed)
    }
    pub fn len(&self) -> usize {
        if let Some(imp) = self.imp.as_ref() {
            imp.len()
        } else {
            0
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Closes the channel.
    /// The remaining envelopes can still be received.
    pub fn close(&self) {
        if let Some(imp) = self.imp.as_ref() {
            imp.close();
        }
    }
    /// Drop self, and closes the channel if there are no senders left.
    pub fn abort(&mut self) {
        self.imp.take();
    }
    /// Sends a envelope into the channel.
    ///
    /// If the envelope is empty, sends flush event to channel if the number of the empty envelopes received is
    /// equal to the number of senders
    ///
    /// If the channel is full, this method waits until there is space for a envelope.
    ///
    /// If the channel is closed, this method returns an error.
    pub async fn send<T>(&self, msg: Envelope<T>) -> Result<(), SendError<Envelope<T>>>
    where
        T: 'static + Send + Clone,
    {
        self.send_any(msg.seal()).await.map_err(|err| {
            let mut envelope = err.0;
            let envelope = envelope
                .downcast_mut::<Envelope<T>>()
                .expect("type error when downcast");
            super::inner::SendError(envelope.take())
        })
    }
    /// Sends a any envelope into the channel. see document of `send` for more detail
    pub async fn send_any(&self, msg: SealedEnvelope) -> Result<(), SendError<SealedEnvelope>> {
        if let Some(imp) = self.imp.as_ref() {
            if msg.is::<DummyEnvelope>() {
                let mut record = self.record.lock().await;
                let epoch = self.epoch.load(Ordering::Relaxed);
                let count = record.entry(epoch).or_insert_with(|| imp.sender_count());
                *count -= 1;

                self.max_epoch.fetch_max(
                    self.epoch.fetch_add(1, Ordering::Relaxed) + 1,
                    Ordering::Relaxed,
                );

                if *count == 0 {
                    record.remove(&epoch);
                    imp.send(msg).await
                } else {
                    Ok(())
                }
            } else {
                self.counter.fetch_add(1, Ordering::Relaxed);
                imp.send(msg).await
            }
        } else {
            Ok(())
        }
    }
}
