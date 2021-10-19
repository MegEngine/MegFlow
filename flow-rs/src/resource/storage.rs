/**
 * \file flow-rs/src/resource/storage.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::*;
use crate::rt::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct StorageInner {
    resources: HashMap<(u64, String), lazy::LazyCell<ResResult>>,
}

impl StorageInner {
    pub(crate) fn get(&mut self, key: &(u64, String)) -> Option<AnyResource> {
        if let Some(resource) = self.resources.get_mut(key) {
            if let Ok(res) = resource.get() {
                return Some(res.clone());
            } else {
                resource.revert();
            }
        } else {
            return None;
        }
        for v in self.resources.values_mut() {
            if let Some(Ok(res)) = v.view() {
                // it is safe because of `&mut self`
                if Arc::strong_count(res) == 1 {
                    v.revert();
                }
            }
        }
        let resource = self.resources.get_mut(key).unwrap();
        if let Ok(res) = resource.get() {
            Some(res.clone())
        } else {
            // FIXME: panic or throw err ?
            None
        }
    }

    pub(crate) fn append(&mut self, other: &mut StorageInner) {
        for (k, v) in std::mem::take(&mut other.resources).into_iter() {
            self.resources.insert(k, v);
        }
    }
}

pub(crate) type ResourceStorage = Mutex<StorageInner>;

pub(crate) fn storage(local_key: u64, id: u64, cfg: &HashMap<String, Entity>) -> ResourceStorage {
    let mut resources = HashMap::new();
    for (name, res) in cfg {
        let slice = ResourceSlice::registry_local()
            .get(local_key)
            .get(&res.ty)
            .unwrap();
        let name_cloned = name.clone();
        let args_cloned = res.args.clone();
        resources.insert(
            (id, name.clone()),
            LazyCell::new(Box::new(move || {
                (slice.cons)(name_cloned.clone(), &args_cloned)
            })),
        );
    }
    Mutex::new(StorageInner { resources })
}
