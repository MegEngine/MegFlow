/**
 * \file flow-rs/src/envelope/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod any_envelope;
#[allow(clippy::module_inception)]
mod envelope;

#[doc(hidden)]
pub use any_envelope::*;
pub use envelope::*;

pub type SealedEnvelope = Box<dyn AnyEnvelope + Send>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basis() {
        let envelope = DummyEnvelope {}.seal();
        assert!(envelope.is::<DummyEnvelope>());
        let envelope = Envelope::new(0f32).seal();
        assert!(!envelope.is::<DummyEnvelope>());
        assert!(envelope.is::<Envelope<f32>>());
    }
}
