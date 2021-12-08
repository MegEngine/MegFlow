/**
 * \file flow-message/cross/cow_map.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::*;

impl private::IndexSetData<String> for CowMap {
    fn set(&mut self, key: String, data: data::Data) {
        self.insert(key, data);
    }
}

impl private::IndexGetDataRefForGet<str> for CowMap {
    fn get_ref(&self, key: &str) -> Option<&data::Data> {
        self.get(key)
    }
}

impl private::IndexGetDataRefForGetRef<str> for CowMap {
    fn get_ref(&self, key: &str) -> Option<&data::Data> {
        self.get(key)
    }
}

impl private::IndexGetDataMut<str> for CowMap {
    fn get_mut(&mut self, key: &str) -> Option<&mut data::Data> {
        self.get_mut(key)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basis() {
        let mut map = CowMap::new();
        map.xset("a".to_owned(), 1i64);
        map.xset("b".to_owned(), 1f64);
        let mut sub = CowMap::new();
        sub.xset("d".to_owned(), false);
        map.xset("c".to_owned(), sub);

        assert_eq!(map.xget("b"), Some(1f64));
        let has_c: Option<&mut CowMap> = map.xget_mut("c");
        assert!(has_c.is_some());
        let sub = has_c.unwrap();
        assert_eq!(sub.xget("d"), Some(false));
        sub.xset("e".to_owned(), true);

        let has_c: Option<&CowMap> = map.xget_ref("c");
        assert!(has_c.is_some());
        let sub = has_c.unwrap();
        assert_eq!(sub.xget("e"), Some(true));
    }
}
