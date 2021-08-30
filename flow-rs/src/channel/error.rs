/**
 * \file flow-rs/src/channel/error.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
pub enum BatchRecvError<T> {
    Closed(Vec<T>),
}

pub type SendError<T> = super::inner::SendError<T>;
pub type RecvError = super::inner::RecvError;

impl<T> std::fmt::Debug for BatchRecvError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BatchRecvError::Closed(_) => f.write_str("BatchRecvError::Closed"),
        }
    }
}
