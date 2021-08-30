/**
 * \file flow-plugins/src/utils/either.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use rweb::openapi::{ResponseEntity, Responses};
use rweb::*;

#[derive(Schema)]
pub(crate) enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Reply for Either<L, R>
where
    L: Reply,
    R: Reply,
{
    #[inline]
    fn into_response(self) -> reply::Response {
        match self {
            Either::Left(l) => l.into_response(),
            Either::Right(r) => r.into_response(),
        }
    }
}

impl<L, R> ResponseEntity for Either<L, R>
where
    L: ResponseEntity,
    R: ResponseEntity,
{
    fn describe_responses() -> Responses {
        let mut lresps = L::describe_responses();
        let rresps = R::describe_responses();

        for (k, rresp) in &rresps {
            if lresps.contains_key(k) {
                let lresp = lresps.get_mut(k).unwrap();
                for (k, v) in &rresp.content {
                    if !lresp.content.contains_key(k) {
                        lresp.content.insert(k.clone(), v.clone());
                    }
                }
            } else {
                lresps.insert(k.clone(), rresp.clone());
            }
        }

        lresps
    }
}
