/**
 * \file flow-plugins/src/utils/error.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use std::fmt::Debug;

use rweb::Rejection;

#[derive(Debug)]
struct RejectCause<T>
where
    T: 'static + Debug + Send + Sync,
{
    err: T,
}

impl<T> rweb::reject::Reject for RejectCause<T> where T: 'static + Debug + Send + Sync {}

impl<T> From<T> for RejectCause<T>
where
    T: 'static + Debug + Send + Sync,
{
    fn from(err: T) -> Self {
        RejectCause { err }
    }
}

pub fn reject_cause<T>(err: T) -> Rejection
where
    T: 'static + Debug + Send + Sync,
{
    rweb::reject::custom(RejectCause::from(err))
}
