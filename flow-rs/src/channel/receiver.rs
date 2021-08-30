/**
 * \file flow-rs/src/channel/receiver.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use super::inner::Receiver as RecvImpl;
use super::{RecvError};
use crate::envelope::{DummyEnvelope, Envelope, SealedEnvelope};

use super::ChannelBase;

#[derive(Default)]
pub struct Receiver {
    imp: Option<RecvImpl<SealedEnvelope>>,
    m_epoch: AtomicUsize,
    g_epoch: Arc<AtomicUsize>,
    is_closed: AtomicBool,
    counter: Arc<AtomicUsize>,
}

impl Clone for Receiver {
    fn clone(&self) -> Self {
        let m_epoch = self.g_epoch.load(Ordering::Relaxed);
        Receiver {
            imp: self.imp.clone(),
            m_epoch: AtomicUsize::new(m_epoch),
            g_epoch: self.g_epoch.clone(),
            is_closed: AtomicBool::new(self.is_closed.load(Ordering::Relaxed)),
            counter: self.counter.clone(),
        }
    }
}

impl ChannelBase for Receiver {
    fn is_closed(&self) -> bool {
        if self.imp.is_some() {
            self.is_closed.load(Ordering::Relaxed)
        } else {
            true
        }
    }
    fn is_none(&self) -> bool {
        self.imp.is_none()
    }
}

impl Receiver {
    pub fn new(
        imp: RecvImpl<SealedEnvelope>,
        epoch: Arc<AtomicUsize>,
        counter: Arc<AtomicUsize>,
    ) -> Self {
        let m_epoch = epoch.load(Ordering::Relaxed);
        Receiver {
            imp: Some(imp),
            m_epoch: AtomicUsize::new(m_epoch),
            g_epoch: epoch,
            is_closed: AtomicBool::new(false),
            counter,
        }
    }
    #[doc(hidden)]
    pub fn empty_n(&self) -> usize {
        self.m_epoch.load(Ordering::Relaxed)
    }
    /// Receives a envelope from the channel.
    ///
    /// If the channel is empty, this method waits until there is a envelope.
    ///
    /// If the channel is closed, this method receives a envelope or returns an error if there are
    /// no more envelopes.
    ///
    /// If the channel encounters a flush event, this method returns an error.
    pub async fn recv<T>(&self) -> Result<Envelope<T>, RecvError>
    where
        T: 'static + Send + Clone,
    {
        self.recv_any().await.map(|mut envelope| {
            let envelope = envelope
                .downcast_mut::<Envelope<T>>()
                .expect("type error when downcast");
            envelope.take()
        })
    }
    /// Receives a any envelope from the channel, see document of `recv` for more detail
    pub async fn recv_any(&self) -> Result<SealedEnvelope, RecvError> {
        let m_epoch = self.m_epoch.load(Ordering::Relaxed);
        let g_epoch = self.g_epoch.load(Ordering::Relaxed);
        if m_epoch < g_epoch {
            self.m_epoch.fetch_add(1, Ordering::Relaxed);
            return Err(RecvError {});
        }
        let imp = self.imp.as_ref().expect("not init");
        let envelope = imp.recv().await.map_err(|e| {
            self.is_closed.store(true, Ordering::Relaxed);
            e
        })?;
        if envelope.is::<DummyEnvelope>() {
            self.m_epoch.fetch_add(1, Ordering::Relaxed);
            self.g_epoch.fetch_add(1, Ordering::Relaxed);
            Err(RecvError {})
        } else {
            self.counter.fetch_add(1, Ordering::Relaxed);
            Ok(envelope)
        }
    }
}
