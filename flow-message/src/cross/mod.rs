#![allow(dead_code)]

mod cow_list;
mod cow_map;
mod data;
mod list;
mod map;
mod python;

use dashmap::DashMap;
use im::{OrdMap, Vector};
use std::sync::Arc;

type Storage<K> = Arc<DashMap<K, data::Data>>;
type CowStorage<K> = OrdMap<K, data::Data>;

pub type CowMap = CowStorage<String>;
pub type Map = Storage<String>;
pub type List = data::Data;
pub type CowList = Vector<data::Data>;

pub trait VectorOpr<T> {
    fn xpush_back(&mut self, value: T);
    fn xpush_front(&mut self, value: T);
    fn xpop_front(&mut self) -> Option<T>;
    fn xpop_back(&mut self) -> Option<T>;
}

pub trait IndexSet<K, T> {
    fn xset(&mut self, key: K, value: T);
}

pub trait IndexGet<Idx, Output>
where
    Output: Copy,
    Idx: ?Sized,
{
    fn xget(&self, i: &Idx) -> Option<Output>;
}

pub trait IndexGetRef<'a, Idx, Output>
where
    Idx: ?Sized,
    Output: 'a,
{
    fn xget_ref(&'a self, i: &Idx) -> Option<Output>;
}

pub trait IndexGetMut<'a, Idx, Output>
where
    Idx: ?Sized,
    Output: 'a,
{
    fn xget_mut(&'a mut self, i: &Idx) -> Option<Output>;
}

#[doc(hidden)]
mod private {
    use super::*;
    pub trait IndexSetData<Idx> {
        fn set(&mut self, key: Idx, value: data::Data);
    }
    pub trait IndexGetDataRefForGet<Idx>
    where
        Idx: ?Sized,
    {
        fn get_ref(&self, key: &'_ Idx) -> Option<&data::Data>;
    }
    pub trait IndexGetDataRefForGetRef<Idx>
    where
        Idx: ?Sized,
    {
        fn get_ref(&self, key: &'_ Idx) -> Option<&data::Data>;
    }
    pub trait IndexGetDataMut<Idx>
    where
        Idx: ?Sized,
    {
        fn get_mut(&mut self, key: &'_ Idx) -> Option<&mut data::Data>;
    }

    impl<Idx, T, U> IndexSet<Idx, U> for T
    where
        T: IndexSetData<Idx>,
        U: Into<data::Data>,
    {
        fn xset(&mut self, key: Idx, value: U) {
            self.set(key, value.into());
        }
    }

    impl<Idx, T, U> IndexGet<Idx, U> for T
    where
        U: Copy,
        T: IndexGetDataRefForGet<Idx>,
        data::Data: data::Get<U>,
        Idx: ?Sized,
    {
        fn xget(&self, key: &Idx) -> Option<U> {
            self.get_ref(key).map(|data| data::Get::get(data))
        }
    }

    impl<'a, Idx, T, U> IndexGetRef<'a, Idx, U> for T
    where
        U: 'a,
        T: IndexGetDataRefForGetRef<Idx>,
        data::Data: data::GetRef<'a, U>,
        Idx: ?Sized,
    {
        fn xget_ref(&'a self, key: &Idx) -> Option<U> {
            self.get_ref(key).map(|data| data::GetRef::get_ref(data))
        }
    }

    impl<'a, Idx, T, U> IndexGetMut<'a, Idx, U> for T
    where
        U: 'a,
        T: IndexGetDataMut<Idx>,
        data::Data: data::GetMut<'a, U>,
        Idx: ?Sized,
    {
        fn xget_mut(&'a mut self, key: &Idx) -> Option<U> {
            self.get_mut(key).map(|data| data::GetMut::get_mut(data))
        }
    }
}
