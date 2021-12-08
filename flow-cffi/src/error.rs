/**
 * \file flow-cffi/error.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::types::MGFStatus;

pub type Result<T> = std::result::Result<T, MGFStatus>;

pub fn load_err(err: anyhow::Error) -> MGFStatus {
    ffi_helpers::update_last_error(err);
    MGFStatus::LoadFault
}

pub fn closed(key: &str) -> MGFStatus {
    ffi_helpers::update_last_error(anyhow::anyhow!("connection[{}] has closed", key));
    MGFStatus::Disconnected
}

pub fn not_found(key: &str) -> MGFStatus {
    ffi_helpers::update_last_error(anyhow::anyhow!("key[{}] is not found", key));
    MGFStatus::NoExistKey
}

pub fn no_running() -> MGFStatus {
    ffi_helpers::update_last_error(anyhow::anyhow!("graph is not running"));
    MGFStatus::NoRunningGraph
}

pub fn nullptr(name: &str) -> MGFStatus {
    ffi_helpers::update_last_error(anyhow::anyhow!("pointer{} is null", name));
    MGFStatus::NullPointer
}
