/**
 * \file flow-rs/src/registry.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

pub trait Collect<ID>: Sized + 'static {
    fn registry_global() -> &'static Registry<ID, Self>;
    fn registry_local() -> &'static RegistryMap<ID, Self>;
}
struct RegistryInner<ID, T> {
    elems: HashMap<ID, T>,
}
pub struct Registry<ID, T> {
    inner: RwLock<RegistryInner<ID, Arc<T>>>,
}
pub struct RegistryMap<ID, T> {
    inner: RwLock<HashMap<u64, Arc<Registry<ID, T>>>>,
}

impl<ID: Eq + Hash, T> Default for Registry<ID, T> {
    fn default() -> Registry<ID, T> {
        Registry {
            inner: RwLock::new(RegistryInner {
                elems: HashMap::new(),
            }),
        }
    }
}

impl<ID: Eq + Hash, T> Default for RegistryMap<ID, T> {
    fn default() -> RegistryMap<ID, T> {
        RegistryMap {
            inner: RwLock::new(Default::default()),
        }
    }
}

impl<ID: Eq + Hash, T> Registry<ID, T> {
    pub(crate) fn get<Q: ?Sized>(&self, id: &Q) -> Option<Arc<T>>
    where
        ID: Borrow<Q>,
        Q: Hash + Eq,
    {
        let registry = self.inner.read().unwrap();
        registry.elems.get(id).cloned()
    }

    pub(crate) fn insert<Q>(&self, id: Q, elem: T)
    where
        Q: Into<ID>,
    {
        let mut registry = self.inner.write().unwrap();
        registry.elems.insert(id.into(), Arc::new(elem));
    }

    pub(crate) fn for_each<F>(&self, f: F)
    where
        F: FnMut(&T),
    {
        let registry = self.inner.read().unwrap();
        registry.elems.values().map(|x| x.as_ref()).for_each(f);
    }

    pub(crate) fn to_vec(&self) -> Vec<Arc<T>> {
        let mut registry = self.inner.write().unwrap();
        let elems = std::mem::take(&mut registry.elems);
        elems.into_iter().map(|(_, v)| v).collect()
    }
}

impl<ID, T> Registry<ID, T>
where
    ID: Hash + Clone + Eq,
{
    pub(crate) fn keys(&self) -> Vec<ID> {
        let registry = self.inner.read().unwrap();
        registry.elems.keys().cloned().collect()
    }
}

impl<ID, T> Clone for Registry<ID, T>
where
    ID: Clone,
{
    fn clone(&self) -> Self {
        let inner = self.inner.read().unwrap();
        Registry {
            inner: RwLock::new(RegistryInner {
                elems: inner.elems.clone(),
            }),
        }
    }
}

type GlobalCallback = RwLock<Vec<Box<dyn Fn(u64) + Send + Sync>>>;

lazy_static::lazy_static! {
    pub(crate) static ref INITIALIZE: GlobalCallback = RwLock::new(vec![]);
}

lazy_static::lazy_static! {
    pub(crate) static ref FINALIZE: GlobalCallback = RwLock::new(vec![]);
}

pub(crate) fn initialize(id: u64) {
    INITIALIZE.read().unwrap().iter().for_each(|f| f(id));
}

pub(crate) fn finalize(id: u64) {
    FINALIZE.read().unwrap().iter().for_each(|f| f(id));
}

impl<ID, T: Collect<ID>> RegistryMap<ID, T>
where
    ID: 'static + Eq + Hash + Clone,
{
    pub(crate) fn __or_insert_only_in_ctor(&self, id: u64) -> Arc<Registry<ID, T>> {
        let mut map = self.inner.write().unwrap();
        map.entry(id)
            .or_insert_with(|| Arc::new(T::registry_global().clone()))
            .clone()
    }

    pub(crate) fn get(&self, id: u64) -> Arc<Registry<ID, T>> {
        let map = self.inner.read().unwrap();
        map.get(&id).cloned().unwrap()
    }

    pub(crate) fn remove(&self, id: u64) {
        let mut map = self.inner.write().unwrap();
        map.remove(&id);
    }
}

pub fn __submit_only_in_ctor<Q, ID, T: Collect<ID>>(key: Q, value: T)
where
    ID: Eq + Hash + 'static,
    Q: Into<ID>,
{
    T::registry_global().insert(key, value);
}

#[macro_export]
#[doc(hidden)]
macro_rules! collect {
    ($id:ty, $ty:ty) => {
        impl $crate::registry::Collect<$id> for $ty {
            #[inline]
            fn registry_global() -> &'static $crate::registry::Registry<$id, Self> {
                lazy_static::lazy_static! {
                    static ref REGISTRY: $crate::registry::Registry<$id, $ty> =
                        $crate::registry::Registry::default();
                };
                &REGISTRY
            }
            #[inline]
            fn registry_local() -> &'static $crate::registry::RegistryMap<$id, Self> {
                lazy_static::lazy_static! {
                    static ref REGISTRY: $crate::registry::RegistryMap<$id, $ty> =
                        $crate::registry::RegistryMap::default();
                };
                #[flow_rs::ctor]
                fn init() {
                    let mut initialize = $crate::registry::INITIALIZE.write().unwrap();
                    initialize.push(Box::new(|id| {
                        REGISTRY.__or_insert_only_in_ctor(id);
                    }));
                    let mut finalize = $crate::registry::FINALIZE.write().unwrap();
                    finalize.push(Box::new(|id| REGISTRY.remove(id)));
                }
                &REGISTRY
            }
        }
    };
}
