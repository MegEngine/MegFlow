/**
 * \file flow-rs/src/debug/protocol.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub ty: String,
    #[serde(flatten, default)]
    pub others: Map<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct RequestMessage {
    pub feature: String,
    pub seq_id: usize,
    pub command: String,
    #[serde(flatten, default)]
    pub args: Map<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct EventMessage {
    pub event: String,
    #[serde(flatten, default)]
    pub args: Map<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseMessage {
    pub success: bool,
    pub feature: String,
    pub seq_id: usize,
    pub command: String,
    #[serde(flatten, default)]
    pub args: Map<String, Value>,
}
// message
pub const TYPE_REQUEST: &str = "request";
pub const TYPE_EVENT: &str = "event";
pub const TYPE_RESPONSE: &str = "response";
// event
pub const EVENT_INITIALIZED: &str = "initialized";
pub const EVENT_TERMINATED: &str = "terminated";
pub const EVENT_STOP: &str = "stop";
// command
pub const CMD_START: &str = "start";
pub const CMD_STOP: &str = "stop";
pub const CMD_NOOP: &str = "noop";
