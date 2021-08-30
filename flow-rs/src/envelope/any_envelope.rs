use super::EnvelopeInfo;
/**
 * \file flow-rs/src/envelope/any_envelope.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use dyn_clone::{clone_trait_object, DynClone};
use std::any::{Any, TypeId};

pub trait AnyEnvelope: Any + DynClone + 'static {
    fn is_some(&self) -> bool;
    fn is_none(&self) -> bool;
    fn info(&self) -> &EnvelopeInfo;
    fn info_mut(&mut self) -> &mut EnvelopeInfo;
}
clone_trait_object!(AnyEnvelope);

impl dyn AnyEnvelope {
    pub fn is<T: AnyEnvelope>(&self) -> bool {
        let t = TypeId::of::<T>();
        let boxed = self.type_id();

        t == boxed
    }

    pub fn downcast_ref<T: AnyEnvelope>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn AnyEnvelope as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: AnyEnvelope>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn AnyEnvelope as *mut T)) }
        } else {
            None
        }
    }
}

impl dyn AnyEnvelope + Send + Sync {
    pub fn is<T: AnyEnvelope>(&self) -> bool {
        <dyn AnyEnvelope>::is::<T>(self)
    }

    pub fn downcast_ref<T: AnyEnvelope>(&self) -> Option<&T> {
        <dyn AnyEnvelope>::downcast_ref::<T>(self)
    }

    pub fn downcast_mut<T: AnyEnvelope>(&mut self) -> Option<&mut T> {
        <dyn AnyEnvelope>::downcast_mut::<T>(self)
    }
}

impl dyn AnyEnvelope + Send {
    pub fn is<T: AnyEnvelope>(&self) -> bool {
        <dyn AnyEnvelope>::is::<T>(self)
    }

    pub fn downcast_ref<T: AnyEnvelope>(&self) -> Option<&T> {
        <dyn AnyEnvelope>::downcast_ref::<T>(self)
    }

    pub fn downcast_mut<T: AnyEnvelope>(&mut self) -> Option<&mut T> {
        <dyn AnyEnvelope>::downcast_mut::<T>(self)
    }
}
