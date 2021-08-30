/**
 * \file flow-rs/src/loader/python/context.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::unlimited::{self, PyThreadStateUnlimited};
use pyo3::ffi;
use pyo3::prelude::*;
use std::cell::RefCell;

thread_local! {
    static CTX: RefCell<ContextPool> = RefCell::new(ContextPool { pool: vec![], freelist: vec![] });
}

pub fn with_context<F, R>(py: Python, f: F) -> R
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    let id = CTX.with(|ctx| ctx.borrow_mut().store());
    let r = py.allow_threads(f);
    CTX.with(|ctx| ctx.borrow_mut().restore(id));
    r
}

struct Context {
    thread: *mut ffi::PyThreadState,
    ctx: PyThreadStateUnlimited,
}

struct ContextPool {
    pool: Vec<Context>,
    freelist: Vec<usize>,
}

impl ContextPool {
    fn store(&mut self) -> usize {
        let id = self.freelist.pop().unwrap_or(self.pool.len());
        if id == self.pool.len() {
            self.pool.push(Context {
                thread: std::ptr::null_mut(),
                ctx: Default::default(),
            })
        }
        unsafe {
            let context = self.pool.get_unchecked_mut(id);
            context.thread = ffi::PyThreadState_Get();
            context.ctx = unlimited::store(context.thread);
        }
        id
    }

    fn restore(&mut self, id: usize) {
        unsafe {
            let context = self.pool.get_unchecked_mut(id);
            ffi::PyThreadState_Swap(context.thread);
            unlimited::restore(context.thread, &context.ctx);
            context.thread = std::ptr::null_mut();
            context.ctx = Default::default();
        }
        self.freelist.push(id);
    }
}
