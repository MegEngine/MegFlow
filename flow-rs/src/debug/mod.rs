/**
 * \file flow-rs/src/debug/mod.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod feature;
mod protocol;
mod server;

use crate::prelude::feature;
pub use protocol::*;
pub use server::{Server, PORT};

feature!(QPS, { ratio: f32 });
