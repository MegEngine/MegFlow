use crate::registry::Collect;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type AnyResource = Arc<dyn 'static + Any + Send + Sync>;
#[doc(hidden)]
pub struct ResourceSlice {
    pub cons: Box<dyn Fn(String, &toml::value::Table) -> AnyResource + Send + Sync>,
}
crate::collect!(String, ResourceSlice);

/// A collection of any resource which has been registered
#[derive(Clone)]
pub struct ResourceCollection {
    resources: HashMap<String, AnyResource>,
    local_key: u64,
}

lazy_static::lazy_static! {
    pub(crate) static ref GLOBAL_RESOURCES: RwLock<HashMap<u64, ResourceCollection>> =
        RwLock::new(Default::default());
}

#[flow_rs::ctor]
fn init() {
    let mut initialize = crate::registry::INITIALIZE.write().unwrap();
    initialize.push(Box::new(|id| {
        let mut resources = GLOBAL_RESOURCES.write().unwrap();
        resources
            .entry(id)
            .or_insert_with(|| ResourceCollection::new(id));
    }));
    let mut finalize = crate::registry::FINALIZE.write().unwrap();
    finalize.push(Box::new(|id| {
        let mut resources = GLOBAL_RESOURCES.write().unwrap();
        resources.remove(&id);
    }));
}

impl ResourceCollection {
    pub(crate) fn new(local_key: u64) -> ResourceCollection {
        ResourceCollection {
            local_key,
            resources: Default::default(),
        }
    }
    pub(crate) fn insert(&mut self, name: String, ty: &str, args: &toml::value::Table) {
        let slice = ResourceSlice::registry_local()
            .get(self.local_key)
            .get(ty)
            .unwrap();
        self.resources
            .insert(name.clone(), (slice.cons)(name, args));
    }
    pub(crate) fn filter(&self, names: &[&str]) -> ResourceCollection {
        let mut resources = HashMap::new();

        for &name in names {
            if let Some(resource) = self.resources.get(name) {
                resources.insert(name.to_owned(), resource.clone());
            }
        }

        ResourceCollection {
            resources,
            local_key: self.local_key,
        }
    }
    pub(crate) fn chain(self, collection: &ResourceCollection) -> ResourceCollection {
        let mut resources = collection.clone();
        for (name, resource) in self.resources {
            resources.resources.entry(name).or_insert(resource);
        }
        resources
    }
    /// Get a resource by name, return None if the type `T` is not match or the name is not exist
    pub fn get<T>(&self, name: &str) -> Option<Arc<T>>
    where
        T: 'static + Any + Send + Sync,
    {
        if let Some(resource) = self.resources.get(name) {
            let cloned = resource.clone();
            cloned.downcast().ok()
        } else {
            None
        }
    }
    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.resources.keys().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    #[test]
    fn test_basis() {
        let local_key = crate::LOCAL_KEY.fetch_add(1, Ordering::Relaxed);
        crate::registry::initialize(local_key);
        ResourceSlice::registry_local().get(local_key).insert(
            "abool_test",
            ResourceSlice {
                cons: Box::new(|_, _| Arc::new(AtomicBool::new(true))),
            },
        );
        let mut collection = ResourceCollection::new(local_key);
        collection.insert(
            "abool_test_instance".to_owned(),
            "abool_test",
            &Default::default(),
        );
        let resource = collection.get::<AtomicBool>("abool_test_instance");
        assert!(resource.is_some());
        let resource = resource.unwrap();
        let is = resource.load(Ordering::Relaxed);
        assert!(is);
        crate::registry::finalize(local_key);
    }
}
