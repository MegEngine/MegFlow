/**
 * \file flow-plugins/src/lib.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod image_server;
mod utils;
mod video_server;

#[doc(hidden)]
pub fn export() {
    pyo3::prepare_freethreaded_python();
    pretty_env_logger::init();
}
