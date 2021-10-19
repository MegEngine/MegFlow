use super::storage::{storage, ResourceStorage};
use super::{AnyResource, Resource};
use crate::config::presentation::Entity;
use std::collections::HashMap;
use std::sync::Arc;

/// A collection of any resource which has been registered
#[derive(Clone, Default)]
pub struct ResourceCollection {
    resources: HashMap<String, u64>,
    storage: Arc<ResourceStorage>,
}

#[derive(Default)]
pub struct UniqueResourceCollection {
    resources: HashMap<String, u64>,
    storage: ResourceStorage,
}

impl UniqueResourceCollection {
    pub(crate) fn new(
        local_key: u64,
        id: u64,
        cfg: &HashMap<String, Entity>,
    ) -> UniqueResourceCollection {
        UniqueResourceCollection {
            resources: cfg.keys().cloned().map(|x| (x, id)).collect(),
            storage: storage(local_key, id, cfg),
        }
    }

    pub(crate) fn take_into_arc(self) -> ResourceCollection {
        ResourceCollection {
            resources: self.resources,
            storage: Arc::new(self.storage),
        }
    }
}

impl ResourceCollection {
    pub(crate) fn filter(&self, names: &[&str]) -> ResourceCollection {
        ResourceCollection {
            resources: names
                .iter()
                .map(|&name| self.resources.get_key_value(name))
                .flatten()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            storage: self.storage.clone(),
        }
    }
    // Chain two ResourceCollections, and `other` will overwrite `self`
    pub(crate) async fn chain(self, mut other: UniqueResourceCollection) -> ResourceCollection {
        let other_storage = other.storage.get_mut();
        self.storage.lock().await.append(other_storage);
        let mut resources = other.resources.clone();
        for (k, v) in self.resources.into_iter() {
            resources.entry(k).or_insert(v);
        }
        ResourceCollection {
            resources,
            storage: self.storage,
        }
    }
    /// Get a resource by name, return None if the type `T` is not match or the name is not exist
    pub async fn get<T>(&self, name: &str) -> Option<Arc<T>>
    where
        T: Resource,
    {
        self.get_any(name)
            .await
            .map(|x| x.downcast_arc().ok())
            .flatten()
    }
    pub async fn get_any(&self, name: &str) -> Option<AnyResource> {
        if let Some((k, v)) = self.resources.get_key_value(name) {
            self.storage.lock().await.get(&(*v, k.clone()))
        } else {
            None
        }
    }
    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.resources.keys().cloned().collect()
    }
}
