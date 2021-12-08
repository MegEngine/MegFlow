/**
 * \file flow-message/cross/data/value.rs
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
    ($ty:ty, $call_ty:ty, $value_type: path) => {
        impl From<$ty> for Data {
            fn from(v: $ty) -> Self {
                let mut data = vec![0;std::mem::size_of::<$ty>()];
                paste! {NativeEndian::[<write_ $call_ty>](&mut data, v as $call_ty)};
                Data {
                    ty: DataType::Value($value_type),
                    data: DataStorage::Bytes(Arc::new(data)),
                }
            }
        }

        impl From<Data> for $ty {
            fn from(data: Data) -> Self {
                match data.ty {
                    DataType::Value($value_type) => {
                        paste! {NativeEndian::[<read_ $call_ty>](data.data.as_bytes().unwrap().as_slice()) as $ty}
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }
    };
    ($ty:ty, $mapper:expr, $value_type: path) => {
        impl From<$ty> for Data {
            fn from(v: $ty) -> Self {
                Data {
                    ty: DataType::Value($value_type),
                    data: DataStorage::Bytes(Arc::new(vec![v as u8])),
                }
            }
        }

        impl From<Data> for $ty {
            fn from(data: Data) -> Self {
                $mapper(data.data.as_bytes().unwrap()[0])
            }
        }
    };
}

impl_from_value_type!(u64, u64, ValueType::Uint);
impl_from_value_type!(i64, i64, ValueType::Int);
impl_from_value_type!(f64, f64, ValueType::Float);
impl_from_value_type!(bool, |x| x != 0, ValueType::Bool);
impl_from_value_type!(u8, |x| x, ValueType::Byte);

macro_rules! impl_get_set {
    ($ty:ty, $call_ty:ty, $value_type: path) => {
        impl Get<$ty> for Data {
            fn get(&self) -> $ty {
                match self.ty {
                    DataType::Value($value_type) => {
                        paste! {NativeEndian::[<read_ $call_ty>](self.data.as_bytes().unwrap().as_slice()) as $ty}
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }
        impl Set<$ty> for Data {
            fn set(&mut self, value: $ty) {
                match self.ty {
                    DataType::Value($value_type) => {
                        paste! {NativeEndian::[<write_ $call_ty>](Arc::make_mut(self.data.as_bytes_mut().unwrap()).as_mut(), value)}
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }
    };
    ($ty:ty, $mapper:expr, $value_type: path) => {
        impl Get<$ty> for Data {
            fn get(&self) -> $ty {
                match self.ty {
                    DataType::Value($value_type) => $mapper(self.data.as_bytes().unwrap()[0]),
                    _ => unreachable!("type is not match"),
                }
            }
        }
        impl Set<$ty> for Data {
            fn set(&mut self, value: $ty) {
                match self.ty {
                    DataType::Value($value_type) => {
                        Arc::make_mut(self.data.as_bytes_mut().unwrap())[0] = value as u8;
                    }
                    _ => unreachable!("type is not match"),
                }
            }
        }
    };
}

impl_get_set!(f64, f64, ValueType::Float);
impl_get_set!(i64, i64, ValueType::Int);
impl_get_set!(u64, u64, ValueType::Uint);
impl_get_set!(bool, |v| v != 0, ValueType::Bool);
impl_get_set!(u8, |v| v, ValueType::Byte);
