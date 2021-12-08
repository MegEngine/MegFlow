/**
 * \file flow-cffi/utils.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::error::*;
use std::ffi::{CStr, OsStr};
use std::path::Path;

pub unsafe fn cstr2path<'a>(s: *const libc::c_char) -> Result<&'a Path> {
    if s.is_null() {
        return Err(nullptr("cstr"));
    }
    let slice = CStr::from_ptr(s);
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::ffi::OsStrExt;
        let osstr = OsStr::from_bytes(slice.to_bytes());
        Ok(osstr.as_ref())
    }
    #[cfg(target_family = "windows")]
    {
        let osstr = std::str::from_utf8(slice.to_bytes()).unwrap();
        Ok(osstr.as_ref())
    }
    #[cfg(target_family = "wasm")]
    {
        unimplemented!()
    }
}

pub unsafe fn cstr2str<'a>(s: *const libc::c_char) -> Result<&'a str> {
    if s.is_null() {
        return Err(nullptr("cstr"));
    }
    let slice = CStr::from_ptr(s);
    Ok(slice.to_str().unwrap())
}
