/**
 * \file flow-cffi/types.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::error::Result;
use super::utils::cstr2path;
use flow_rs::loader::*;
use std::convert::TryFrom;

#[repr(C)]
pub enum MGFStatus {
    Success,
    LoadFault,
    Disconnected,
    NullPointer,
    NoExistKey,
    NoRunningGraph,
    Internal,
}

#[derive(Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub enum MGFPluginType {
    Python,
}

#[repr(C)]
pub struct MGFLoaderConfig {
    plugin_path: *const libc::c_char,
    module_path: *const libc::c_char,
    ty: MGFPluginType,
}

#[repr(C)]
pub struct MGFMessage {
    ptr: *mut libc::c_void,
    clone_func: extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
    release_func: extern "C" fn(*mut libc::c_void),
}

pub type MGFGraph = *mut libc::c_void;

impl TryFrom<MGFLoaderConfig> for LoaderConfig {
    type Error = crate::types::MGFStatus;
    fn try_from(cfg: MGFLoaderConfig) -> Result<Self> {
        assert_eq!(cfg.ty, MGFPluginType::Python);
        Ok(LoaderConfig {
            plugin_path: unsafe { cstr2path(cfg.plugin_path)? }.into(),
            module_path: unsafe { cstr2path(cfg.module_path)? }.into(),
            ty: PluginType::Python,
        })
    }
}

#[cfg(test)]
mod test {
    use super::MGFMessage;
    use flow_message::c::CMessage;
    #[test]
    fn test_mgf_message_field() {
        assert_eq!(
            std::mem::size_of::<MGFMessage>(),
            std::mem::size_of::<CMessage>()
        );
    }
}
