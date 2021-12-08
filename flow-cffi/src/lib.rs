#![allow(clippy::missing_safety_doc)]

/**
 * \file flow-cffi/lib.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod error;
mod graph;
mod types;
mod utils;

use flow_message::c::*;
use flow_rs::prelude::*;
use std::convert::TryFrom;
use std::ffi::CString;

lazy_static::lazy_static! {
    static ref VERSION: CString = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
}

macro_rules! c_throw {
    ($call: expr) => {
        match $call {
            Ok(ret) => ret,
            Err(e) => return e,
        }
    };
}

/// Get the version of the libflow_cffi.so, e.g. "1.0.0"
#[no_mangle]
pub unsafe extern "C" fn MGF_version() -> *const libc::c_char {
    VERSION.as_ptr() as *const _
}

/// Load a graph.
#[no_mangle]
pub unsafe extern "C" fn MGF_load_graph(
    config_path: *const libc::c_char,
    cgraph: &mut types::MGFGraph,
) -> types::MGFStatus {
    *cgraph = std::ptr::null_mut();
    let path = c_throw!(utils::cstr2path(config_path));
    let graph = c_throw!(graph::Graph::load(None, path));
    *cgraph = Box::into_raw(Box::new(graph)) as *mut _;
    types::MGFStatus::Success
}

/// Load a graph with plugins.
#[no_mangle]
pub unsafe extern "C" fn MGF_load_graph_with_plugins(
    config_path: *const libc::c_char,
    plugin_option: types::MGFLoaderConfig,
    cgraph: &mut types::MGFGraph,
) -> types::MGFStatus {
    *cgraph = std::ptr::null_mut();
    let path = c_throw!(utils::cstr2path(config_path));
    let option = c_throw!(flow_rs::loader::LoaderConfig::try_from(plugin_option));
    let graph = c_throw!(graph::Graph::load(Some(option), path));
    *cgraph = Box::into_raw(Box::new(graph)) as *mut _;
    types::MGFStatus::Success
}

/// Start the graph.
/// # Safety
/// This function is not thread safe
#[no_mangle]
pub unsafe extern "C" fn MGF_start_graph(graph: types::MGFGraph) -> types::MGFStatus {
    let graph = c_throw!((graph as *mut graph::Graph)
        .as_mut()
        .ok_or_else(|| error::nullptr("graph")));
    c_throw!(graph.start());
    types::MGFStatus::Success
}

/// Close and wait for the graph to run to completion.
/// # Safety
/// 1. This function will free the graph at the end of the function.
/// 2. This function is not thread safe
#[no_mangle]
pub unsafe extern "C" fn MGF_close_and_wait_graph(graph: types::MGFGraph) -> types::MGFStatus {
    let p = graph as *mut graph::Graph;
    let graph = c_throw!(p.as_mut().ok_or_else(|| error::nullptr("graph")));
    c_throw!(rt::task::block_on(async { graph.close_and_wait().await }));
    let _ = Box::from_raw(p);
    types::MGFStatus::Success
}

/// Send a message to a specific port of the graph
#[no_mangle]
pub unsafe extern "C" fn MGF_send_message(
    graph: types::MGFGraph,
    name: *const libc::c_char,
    message: types::MGFMessage,
) -> types::MGFStatus {
    let name = c_throw!(utils::cstr2str(name));
    let graph = (graph as *mut graph::Graph).as_mut().unwrap();
    let message: CMessage = std::mem::transmute(message);
    c_throw!(rt::task::block_on(async {
        graph.send(name, Message::from(message)).await
    }));
    types::MGFStatus::Success
}

/// Receiver a message from a specific port of the graph
#[no_mangle]
pub unsafe extern "C" fn MGF_recv_message(
    graph: types::MGFGraph,
    name: *const libc::c_char,
    cmessage: &mut types::MGFMessage,
) -> types::MGFStatus {
    let name = c_throw!(utils::cstr2str(name));
    let graph = c_throw!((graph as *mut graph::Graph)
        .as_mut()
        .ok_or_else(|| error::nullptr("graph")));
    let message = c_throw!(rt::task::block_on(async { graph.recv(name).await }));
    *cmessage = std::mem::transmute(CMessage::from(message));
    types::MGFStatus::Success
}

/// Clear the last error.
#[no_mangle]
pub unsafe extern "C" fn MGF_clear_last_error() {
    ffi_helpers::error_handling::clear_last_error()
}

/// Get the length of the last error message in bytes when encoded as UTF-8,
/// including the trailing null.
#[no_mangle]
pub unsafe extern "C" fn MGF_last_error_length() -> libc::c_int {
    ffi_helpers::error_handling::last_error_length()
}

/// Peek at the most recent error and write its error message
/// into the provided buffer as a UTF-8 encoded string.
///
/// This returns the number of bytes written, or `-1` if there was an error.
#[no_mangle]
pub unsafe extern "C" fn MGF_error_message(
    buf: *mut libc::c_char,
    length: libc::c_int,
) -> libc::c_int {
    ffi_helpers::error_handling::error_message_utf8(buf, length)
}
