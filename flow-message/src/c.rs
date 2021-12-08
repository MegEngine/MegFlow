/**
 * \file flow-message/c.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */

#[repr(C)]
pub struct CMessage {
    pub ptr: *mut libc::c_void,
    pub clone_func: extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
    pub release_func: extern "C" fn(*mut libc::c_void),
}

pub struct Message(Option<CMessage>);

unsafe impl Send for Message {}
unsafe impl Sync for Message {}

impl Message {
    pub fn as_ptr<T>(&self) -> *const T {
        self.0.as_ref().unwrap().ptr as *const _
    }

    pub fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.0.as_mut().unwrap().ptr as *mut T
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        if let Some(inner) = self.0.as_ref() {
            (inner.release_func)(inner.ptr)
        }
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        let inner = self.0.as_ref().unwrap();
        Message(Some(CMessage {
            ptr: (inner.clone_func)(inner.ptr),
            clone_func: inner.clone_func,
            release_func: inner.release_func,
        }))
    }
}

impl From<CMessage> for Message {
    fn from(msg: CMessage) -> Self {
        Message(Some(msg))
    }
}

impl From<Message> for CMessage {
    fn from(mut msg: Message) -> Self {
        msg.0.take().unwrap()
    }
}
