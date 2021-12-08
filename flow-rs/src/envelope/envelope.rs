/**
 * \file flow-rs/src/envelope/envelope.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::{AnyEnvelope, SealedEnvelope};
use std::any::Any;
use std::sync::Arc;

/// `EnvelopeInfo` is a type that represents common information for a message
#[derive(Default, Clone)]
pub struct EnvelopeInfo {
    /// Sequence id, and could be repeat
    pub partial_id: Option<u64>,
    /// Address where the envelope comes from
    pub from_addr: Option<u64>,
    /// Address where the envelope send to
    pub to_addr: Option<u64>,
    /// Address where the envelope transfer to
    pub transfer_addr: Option<u64>,
    /// A tag to mark dynamic graph which the envelope send to
    pub tag: Option<String>,
    /// Extra data
    pub extra_data: Option<Arc<dyn Any + Send + Sync>>,
}
/// `Envelope<M>` is a type that contains a info:`EnvelopeInfo` and a message:`M`
///
/// # Examples
///
/// ```rust
/// # use std::error::Error;
/// # use flow_rs::envelope::*;
/// # fn main() -> Result<(), Box<dyn Error>>{
/// let envelope = Envelope::new(1usize);
/// let mut sealed_envelope = envelope.seal();
/// assert!(sealed_envelope.downcast_mut::<Envelope<usize>>().is_some());
/// let mut envelope = sealed_envelope.downcast_mut::<Envelope<usize>>().unwrap().take();
/// let msg = envelope.unpack();
/// assert_eq!(msg, 1);
/// #
/// #     Ok(())
/// # }
/// ```
pub struct Envelope<M> {
    info: Arc<EnvelopeInfo>, // cow
    msg: Option<M>,
}

impl<M> Envelope<M>
where
    M: 'static + Send + Clone,
{
    /// Create a envelope with message
    pub fn new(msg: M) -> Envelope<M> {
        Self::with_info(msg, Default::default())
    }
    pub fn with_info(msg: M, info: EnvelopeInfo) -> Envelope<M> {
        Envelope {
            msg: Some(msg),
            info: Arc::new(info),
        }
    }
    /// Create a envelope without message
    pub fn empty() -> Envelope<M> {
        Envelope {
            msg: None,
            info: Default::default(),
        }
    }
    /// Convert the `Envelope` to `SealedEnvelope`
    pub fn seal(self) -> SealedEnvelope {
        Box::new(self)
    }
    /// Take the message from the envelope
    ///
    /// # Panics
    ///
    /// If the envelope has no message
    pub fn unpack(&mut self) -> M {
        self.msg.take().expect("message not found")
    }
    /// Repack a message, and return a new envelope with the message
    pub fn repack<T>(&self, msg: T) -> Envelope<T> {
        Envelope {
            msg: Some(msg),
            info: self.info.clone(),
        }
    }
    /// Repack a message in place
    pub fn repack_inplace(&mut self, msg: M) {
        self.msg = Some(msg);
    }
    /// Takes the message out of the envelope, leaving a `None` in its place
    pub fn take(&mut self) -> Envelope<M> {
        Envelope {
            msg: self.msg.take(),
            info: self.info.clone(),
        }
    }
    /// Return a reference to the message
    ///
    /// # Panics
    ///
    /// If the envelope has no message
    pub fn get_ref(&self) -> &M {
        self.msg.as_ref().expect("message not found")
    }
    /// Return a mutable reference to the message
    ///
    /// # Panics
    ///
    /// If the envelope has no message
    pub fn get_mut(&mut self) -> &mut M {
        self.msg.as_mut().expect("message not found")
    }
}

impl<M> Clone for Envelope<M>
where
    M: Clone,
{
    fn clone(&self) -> Self {
        Envelope {
            msg: self.msg.clone(),
            info: self.info.clone(),
        }
    }
}

impl<M> AnyEnvelope for Envelope<M>
where
    M: 'static + Clone + Send,
{
    fn is_none(&self) -> bool {
        self.msg.is_none()
    }
    fn is_some(&self) -> bool {
        self.msg.is_some()
    }
    fn info(&self) -> &EnvelopeInfo {
        &self.info
    }
    fn info_mut(&mut self) -> &mut EnvelopeInfo {
        Arc::make_mut(&mut self.info)
    }
}

#[doc(hidden)]
#[derive(Clone)]
pub struct DummyEnvelope;

impl DummyEnvelope {
    pub fn seal(self) -> SealedEnvelope {
        Box::new(self)
    }
}

impl AnyEnvelope for DummyEnvelope {
    fn is_none(&self) -> bool {
        true
    }
    fn is_some(&self) -> bool {
        false
    }
    fn info(&self) -> &EnvelopeInfo {
        unimplemented!()
    }
    fn info_mut(&mut self) -> &mut EnvelopeInfo {
        unimplemented!()
    }
}
