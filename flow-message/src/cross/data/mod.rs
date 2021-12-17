/**
 * \file flow-message/cross/data/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod list;
mod value;

use im::Vector;

use super::*;
use enum_as_inner::EnumAsInner;

pub trait IndexGet<Idx, Output>
where
    Idx: ?Sized,
{
    fn get(&self, index: Idx) -> Option<Output>;
}

pub trait IndexSet<Idx, Value>
where
    Idx: ?Sized,
{
    fn set(&mut self, index: Idx, value: Value);
}

pub trait Get<Output> {
    fn get(&self) -> Output;
}

pub trait Set<Value> {
    fn set(&mut self, value: Value);
}

pub trait GetRef<'a, Output>
where
    Output: 'a,
{
    fn get_ref(&'a self) -> Output;
}

pub trait GetMut<'a, Output>
where
    Output: 'a,
{
    fn get_mut(&'a mut self) -> Output;
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Float,
    Int,
    Uint,
    Byte,
    Bool,
}

impl ValueType {
    fn width(&self) -> usize {
        match *self {
            ValueType::Float | ValueType::Int | ValueType::Uint => 8,
            ValueType::Byte | ValueType::Bool => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CollectionType {
    List(ValueType),
    CowList,
    Map,
    CowMap,
}

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Value(ValueType),
    Collection(CollectionType),
}

#[derive(EnumAsInner, Clone)]
pub enum DataStorage {
    Bytes(Arc<Vec<u8>>), // immutable bytes, List<Value> or Value
    Datas(CowList),      // CowList<Any>
    Ref(Map),            // Map<String, Any>
    CowRef(CowMap),      // CowMap<String, Any>
}

#[derive(Clone)]
pub struct Data {
    ty: DataType,
    data: DataStorage,
}

impl Data {
    pub fn ty(&self) -> DataType {
        self.ty
    }

    pub fn data(&self) -> &DataStorage {
        &self.data
    }

    pub fn len(&self) -> usize {
        match self.ty {
            DataType::Value(_) => 1usize,
            DataType::Collection(CollectionType::List(value_ty)) => {
                self.data.as_bytes().unwrap().len() / value_ty.width()
            }
            DataType::Collection(CollectionType::Map) => self.data.as_ref().unwrap().len(),
            DataType::Collection(CollectionType::CowList) => self.data.as_datas().unwrap().len(),
            DataType::Collection(CollectionType::CowMap) => self.data.as_cow_ref().unwrap().len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> GetRef<'a, &'a Map> for Data {
    fn get_ref(&self) -> &Map {
        match self.ty {
            DataType::Collection(CollectionType::Map) => self.data.as_ref().unwrap(),
            _ => unreachable!("type is not match"),
        }
    }
}

impl<'a> GetRef<'a, &'a CowList> for Data {
    fn get_ref(&self) -> &Vector<Data> {
        match self.ty {
            DataType::Collection(CollectionType::CowList) => self.data.as_datas().unwrap(),
            _ => unreachable!("type is not match"),
        }
    }
}

impl<'a> GetRef<'a, &'a CowMap> for Data {
    fn get_ref(&self) -> &CowStorage<String> {
        match self.ty {
            DataType::Collection(CollectionType::CowMap) => self.data.as_cow_ref().unwrap(),
            _ => unreachable!("type is not match"),
        }
    }
}

impl<'a> GetMut<'a, &'a mut Map> for Data {
    fn get_mut(&mut self) -> &mut Map {
        match self.ty {
            DataType::Collection(CollectionType::Map) => self.data.as_ref_mut().unwrap(),
            _ => unreachable!("type is not match"),
        }
    }
}

impl<'a> GetMut<'a, &'a mut CowList> for Data {
    fn get_mut(&mut self) -> &mut Vector<Data> {
        match self.ty {
            DataType::Collection(CollectionType::CowList) => self.data.as_datas_mut().unwrap(),
            _ => unreachable!("type is not match"),
        }
    }
}

impl<'a> GetMut<'a, &'a mut CowMap> for Data {
    fn get_mut(&mut self) -> &mut CowStorage<String> {
        match self.ty {
            DataType::Collection(CollectionType::CowMap) => self.data.as_cow_ref_mut().unwrap(),
            _ => unreachable!("type is not match"),
        }
    }
}

impl From<CowMap> for Data {
    fn from(r: CowMap) -> Self {
        Self {
            ty: DataType::Collection(CollectionType::CowMap),
            data: DataStorage::CowRef(r),
        }
    }
}

impl From<Map> for Data {
    fn from(r: Map) -> Self {
        Self {
            ty: DataType::Collection(CollectionType::Map),
            data: DataStorage::Ref(r),
        }
    }
}

impl From<CowList> for Data {
    fn from(datas: CowList) -> Self {
        Self {
            ty: DataType::Collection(CollectionType::CowList),
            data: DataStorage::Datas(datas),
        }
    }
}

impl From<Data> for CowMap {
    fn from(data: Data) -> Self {
        data.data.into_cow_ref().map_err(|_| ()).unwrap()
    }
}

impl From<Data> for Map {
    fn from(data: Data) -> Self {
        data.data.into_ref().map_err(|_| ()).unwrap()
    }
}

impl From<Data> for CowList {
    fn from(data: Data) -> Self {
        data.data.into_datas().map_err(|_| ()).unwrap()
    }
}
