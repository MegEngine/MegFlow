/**
 * \file flow-message/cross/map.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::data::GetRef;
use super::*;
use dashmap::mapref::one::Ref;

pub struct MapRef<'a> {
    inner: Ref<'a, String, data::Data>,
}

impl<'a> MapRef<'a> {
    fn new(inner: Ref<'a, String, data::Data>) -> Self {
        MapRef { inner }
    }
}

impl<'a> AsRef<List> for MapRef<'a> {
    fn as_ref(&self) -> &List {
        self.inner.value()
    }
}

impl<'a> AsRef<Map> for MapRef<'a> {
    fn as_ref(&self) -> &Map {
        GetRef::get_ref(self.inner.value())
    }
}

impl<'a> AsRef<CowMap> for MapRef<'a> {
    fn as_ref(&self) -> &CowMap {
        GetRef::get_ref(self.inner.value())
    }
}

impl private::IndexSetData<String> for Map {
    fn set(&mut self, key: String, data: data::Data) {
        self.as_ref().insert(key, data);
    }
}

impl<T> IndexGet<str, T> for Map
where
    T: Copy,
    data::Data: data::Get<T>,
{
    fn xget(&self, key: &str) -> Option<T> {
        self.as_ref()
            .get(key)
            .map(|data| data::Get::get(data.value()))
    }
}

impl<'a> IndexGetRef<'a, str, MapRef<'a>> for Map {
    fn xget_ref(&'a self, key: &str) -> Option<MapRef<'a>> {
        self.as_ref().get(key).map(MapRef::new)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basis() {
        let mut map: Map = Default::default();
        map.xset("a".to_owned(), 1i64);
        map.xset("b".to_owned(), 1f64);
        let mut sub: Map = Default::default();
        sub.xset("d".to_owned(), false);
        map.xset("c".to_owned(), sub);

        assert_eq!(map.xget("b"), Some(1f64));
        let has_c = map.xget_ref("c");
        assert!(has_c.is_some());
        let elem_ref = has_c.unwrap();
        let sub: &Map = elem_ref.as_ref();
        assert_eq!(sub.xget("d"), Some(false));
    }
}
