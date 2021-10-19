/**
 * \file flow-rs/src/resource/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod any_resource;
mod collection;
mod lazy;
mod storage;

use crate::config::presentation::Entity;
use crate::registry::Collect;
use any_resource::*;
use lazy::LazyCell;

pub use any_resource::Resource;
pub use collection::*;

type ResResult = Result<AnyResource, std::io::Error>;

#[doc(hidden)]
pub struct ResourceSlice {
    pub cons: Box<dyn Fn(String, &toml::value::Table) -> ResResult + Send + Sync>,
}
crate::collect!(String, ResourceSlice);

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    impl Resource for AtomicBool {
        fn to_python(&self, _: pyo3::Python) -> pyo3::PyObject {
            unimplemented!()
        }
    }

    #[crate::rt::test]
    async fn test_basis() {
        let local_key = crate::LOCAL_KEY.fetch_add(1, Ordering::Relaxed);
        crate::registry::initialize(local_key);
        ResourceSlice::registry_local().get(local_key).insert(
            "abool_test",
            ResourceSlice {
                cons: Box::new(|_, _| Ok(Arc::new(AtomicBool::new(true)))),
            },
        );
        let mut map = HashMap::new();
        map.insert(
            "abool_test_instance".to_owned(),
            Entity {
                name: "abool_test_instance".to_owned(),
                ty: "abool_test".to_owned(),
                args: Default::default(),
            },
        );
        let collection = UniqueResourceCollection::new(local_key, 0, &map).take_into_arc();
        let resource = collection.get::<AtomicBool>("abool_test_instance").await;
        assert!(resource.is_some());
        let resource = resource.unwrap();
        let is = resource.load(Ordering::Relaxed);
        assert!(is);
        crate::registry::finalize(local_key);
    }
}
