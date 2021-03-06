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
mod utils;
#[cfg(feature = "python")]
#[path = "./"]
mod python {
    #[cfg(feature = "default")]
    mod bytes_server;
    #[cfg(feature = "default")]
    mod image_input;
    #[cfg(feature = "default")]
    mod image_server;
    #[cfg(feature = "default")]
    mod video_input;
    #[cfg(feature = "default")]
    mod video_server;

    pub(super) fn export() {
        pyo3::prepare_freethreaded_python();
    }
}

#[doc(hidden)]
pub fn export() {
    pretty_env_logger::init();
    #[cfg(feature = "python")]
    python::export();
}
