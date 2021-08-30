/**
 * \file flow-rs/src/channel/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod error;
mod inner;
mod receiver;
mod sender;
mod storage;

pub trait ChannelBase {
    fn is_closed(&self) -> bool;
    fn is_none(&self) -> bool;
}

pub use error::*;
pub use receiver::*;
pub use sender::*;
pub use storage::*;

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::DummyEnvelope;
    use crate::rt;

    #[rt::test]
    async fn test_dummy_msg() {
        let chan = ChannelStorage::unbound();
        assert_eq!(chan.sender_count(), 0);
        assert_eq!(chan.receiver_count(), 0);
        let s1 = chan.sender();
        let s2 = chan.sender();
        let r1 = chan.receiver();
        let r2 = chan.receiver();
        assert_eq!(chan.sender_count(), 2);
        assert_eq!(chan.receiver_count(), 2);
        let _ = chan.clone();
        assert_eq!(chan.sender_count(), 2);
        assert_eq!(chan.receiver_count(), 2);

        s1.send_any(DummyEnvelope {}.seal()).await.ok();
        s2.send_any(DummyEnvelope {}.seal()).await.ok();
        assert_eq!(chan.len(), 1);

        assert!(r1.recv_any().await.is_err());
        assert!(!r1.is_closed());
        assert!(r2.recv_any().await.is_err());
        assert!(!r2.is_closed());

        s2.send_any(DummyEnvelope {}.seal()).await.ok();
        s2.send_any(DummyEnvelope {}.seal()).await.ok();
        assert!(chan.is_empty());
    }

    #[rt::test]
    async fn test_closed() {
        let chan = ChannelStorage::unbound();
        let s = chan.sender();
        let r = chan.receiver();
        s.close();
        assert!(!r.is_closed());
        assert!(r.recv_any().await.is_err());
        assert!(r.is_closed());

        let chan = ChannelStorage::unbound();
        let s = chan.sender();
        let r = chan.receiver();
        chan.close();
        assert!(!r.is_closed());
        assert!(r.recv_any().await.is_err());
        assert!(r.is_closed());
        drop(s);
    }
}
