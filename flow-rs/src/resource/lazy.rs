/**
 * \file flow-rs/src/resource/lazy.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */

// Async initialized lazily
pub struct LazyCell<T> {
    cons: Box<dyn Fn() -> T + Send + Sync>,
    inner: Option<T>,
}

impl<T> LazyCell<T> {
    pub fn new(cons: Box<dyn Fn() -> T + Send + Sync>) -> LazyCell<T> {
        LazyCell { cons, inner: None }
    }

    pub fn revert(&mut self) {
        self.inner = None;
    }

    pub fn get(&mut self) -> &T {
        if self.inner.is_none() {
            self.inner = Some((self.cons)());
        }
        self.inner.as_ref().unwrap()
    }

    pub fn view(&self) -> Option<&T> {
        self.inner.as_ref()
    }
}

// we will lock when we set UnsafeCell<T>
unsafe impl<T> Sync for LazyCell<T> where T: Send + Sync {}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    #[crate::rt::test]
    async fn test_basis() {
        let cap = 100;
        let mut raw = LazyCell::new(Box::new(move || Arc::new(Vec::<usize>::with_capacity(cap))));
        assert!(raw.view().is_none());
        let has_init = raw.get();
        assert_eq!(has_init.capacity(), cap);
        assert!(raw.view().is_some());
    }
}
