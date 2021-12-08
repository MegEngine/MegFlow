/**
 * \file flow-message/cross/data/list.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::*;
use byteorder::{ByteOrder, NativeEndian};
use paste::paste;
use std::convert::From;
use std::sync::Arc;

macro_rules! impl_from_value_type {
    ($ty:ty, $value_type: path) => {
        impl From<Vec<$ty>> for Data {
            fn from(mut src: Vec<$ty>) -> Self {
                let length = src.len();
                let capacity = src.capacity();
                let ptr = src.as_mut_ptr();
                let byte_n = std::mem::size_of::<$ty>();
                src.leak();
                Self {
                    ty: DataType::Collection(CollectionType::List($value_type)),
                    data: DataStorage::Bytes(Arc::new(unsafe {
                        Vec::from_raw_parts(ptr as *mut u8, length * byte_n, capacity * byte_n)
                    })),
                }
            }
        }

        impl From<Data> for Vec<$ty> {
            fn from(src: Data) -> Self {
                match src.ty {
                    DataType::Collection(CollectionType::List($value_type)) => {
                        let mut bytes = src
                            .data
                            .into_bytes()
                            .map_err(|_| ())
                            .unwrap()
                            .as_ref()
                            .clone();
                        let length = bytes.len() / std::mem::size_of::<$ty>();
                        let capacity = bytes.capacity() / std::mem::size_of::<$ty>();
                        let ptr = bytes.as_mut_ptr();
                        bytes.leak();
                        unsafe { Vec::from_raw_parts(ptr as *mut $ty, length, capacity) }
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }
    };
}

impl_from_value_type!(f64, ValueType::Float);
impl_from_value_type!(i64, ValueType::Int);
impl_from_value_type!(u64, ValueType::Uint);
impl_from_value_type!(u8, ValueType::Byte);
impl_from_value_type!(bool, ValueType::Bool);

macro_rules! impl_index_get {
    ($ty:ty, $call_ty:ty, $value_type: path) => {
        impl IndexGet<usize, $ty> for Data {
            fn get(&self, index: usize) -> Option<$ty> {
                match self.ty {
                    DataType::Collection(CollectionType::List($value_type)) => {
                        let index = index * std::mem::size_of::<$call_ty>();
                        let bytes = self.data.as_bytes().unwrap();
                        if bytes.len() >= index + std::mem::size_of::<$call_ty>() {
                            Some(paste! {NativeEndian::[<read_ $call_ty>](&bytes[index..]) as $ty})
                        } else {
                            None
                        }
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }

        impl IndexSet<usize, $ty> for Data {
            fn set(&mut self, index: usize, value: $ty) {
                match self.ty {
                    DataType::Collection(CollectionType::List($value_type)) => {
                        let index = index * std::mem::size_of::<$call_ty>();
                        let bytes = self.data.as_bytes_mut().unwrap();
                        paste! {NativeEndian::[<write_ $call_ty>](&mut Arc::make_mut(bytes)[index..], value)};
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }
    };
    ($ty:ty, $mapper:expr, $value_type: path) => {
        impl IndexGet<usize, $ty> for Data {
            fn get(&self, index: usize) -> Option<$ty> {
                match self.ty {
                    DataType::Collection(CollectionType::List($value_type)) => {
                        self.data.as_bytes().unwrap().get(index).map($mapper)
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }

        impl IndexSet<usize, $ty> for Data {
            fn set(&mut self, index: usize, value: $ty) {
                match self.ty {
                    DataType::Collection(CollectionType::List($value_type)) => {
                        let bytes = self.data.as_bytes_mut().unwrap();
                        Arc::make_mut(bytes)[index] = value as u8;
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }
    };
}

impl_index_get!(f64, f64, ValueType::Float);
impl_index_get!(i64, i64, ValueType::Int);
impl_index_get!(u64, u64, ValueType::Uint);
impl_index_get!(u8, |v| *v, ValueType::Byte);
impl_index_get!(bool, |v| *v != 0, ValueType::Bool);
