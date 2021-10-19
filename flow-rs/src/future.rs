/**
 * \file flow-rs/src/future.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use futures_util::stream::{FuturesUnordered, StreamExt};
use std::future::Future;

// https://users.rust-lang.org/t/usage-of-futures-select-ok/46068/10
pub(crate) async fn select_ok<F, A, B>(futs: impl IntoIterator<Item = F>) -> Result<A, B>
where
    F: Future<Output = Result<A, B>>,
{
    let mut futs: FuturesUnordered<F> = futs.into_iter().collect();

    let mut last_error: Option<B> = None;
    while let Some(next) = futs.next().await {
        match next {
            Ok(ok) => return Ok(ok),
            Err(err) => {
                last_error = Some(err);
            }
        }
    }
    Err(last_error.expect("Empty iterator."))
}

#[cfg(test)]
mod test {
    use super::*;
    use flow_rs::rt::channel::unbounded;

    #[flow_rs::rt::test]
    async fn test_basis() {
        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();

        s1.send(1).await.ok();
        s2.send(2).await.ok();

        let fut = vec![r1.recv(), r2.recv()];
        assert_eq!(select_ok(fut).await, Ok(1));
        let fut = vec![r1.recv(), r2.recv()];
        assert_eq!(select_ok(fut).await, Ok(2));

        s1.send(1).await.ok();
        let fut = vec![r1.recv(), r2.recv()];
        assert_eq!(select_ok(fut).await, Ok(1));

        drop(s1);
        drop(s2);

        let fut = vec![r1.recv(), r2.recv()];
        assert!(select_ok(fut).await.is_err());
    }
}
