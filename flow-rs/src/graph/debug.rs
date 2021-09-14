/**
 * \file flow-rs/src/graph/debug.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::Graph;
use crate::debug::*;
use crate::rt::task::JoinHandle;
use futures_util::{pin_mut, select, FutureExt};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct NodeQps {
    name: String,
    qps: HashMap<String, (usize, usize)>, // size, qps
    is_block: bool,
}

impl Graph {
    pub(super) fn dmon(&self) -> JoinHandle<()> {
        let conns: Vec<_> = self.conns.values().cloned().collect();
        let ctx = self.ctx.clone();
        let mut first = true; // drop qps result fetched first
        crate::rt::task::spawn(async move {
            let wait_graph = ctx.wait().fuse();
            pin_mut!(wait_graph);
            'start: while !ctx.is_closed() {
                if !QPS.enable() {
                    first = true;
                }
                let wait_qps = QPS.wait().fuse();
                pin_mut!(wait_qps);
                select! {
                    _ = wait_qps => {},
                    _ = wait_graph => break 'start,
                }

                let args = QPS_args
                    .read()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (*k, v.ratio))
                    .next();
                if let Some((seq_id, ratio)) = args {
                    let div_ratio = 1f32 / ratio;
                    let mut nodes = HashMap::new();

                    loop {
                        for conn in &conns {
                            let is_block = conn.get().is_almost_full();
                            let size = conn.get().len();
                            let tx_qps = conn.get().swap_tx_counter() as f32 * ratio;
                            let rx_qps = conn.get().swap_rx_counter() as f32 * ratio;
                            if first {
                                continue;
                            }
                            for tx in &conn.info().tx {
                                let qps = nodes.entry(tx.node_name.clone()).or_insert(NodeQps {
                                    name: tx.node_name.clone(),
                                    qps: Default::default(),
                                    is_block: false,
                                });
                                qps.qps
                                    .insert(tx.port_name.clone(), (size, tx_qps as usize));
                            }
                            for rx in &conn.info().rx {
                                let qps = nodes.entry(rx.node_name.clone()).or_insert(NodeQps {
                                    name: rx.node_name.clone(),
                                    qps: Default::default(),
                                    is_block: false,
                                });
                                qps.qps
                                    .insert(rx.port_name.clone(), (size, rx_qps as usize));
                                qps.is_block = qps.is_block || is_block;
                            }
                        }
                        if first {
                            crate::rt::task::sleep(std::time::Duration::from_secs(
                                (1f32 * div_ratio) as u64,
                            ))
                            .await;
                            first = false;
                        } else {
                            break;
                        }
                    }

                    let into_object = |json| match json {
                        serde_json::Value::Object(json) => json,
                        _ => unreachable!(),
                    };

                    let args = into_object(serde_json::json!({
                        "graph": ctx.ty,
                        "nodes": nodes.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
                    }));

                    let others = into_object(
                        serde_json::to_value(ResponseMessage {
                            success: true,
                            feature: "QPS".to_owned(),
                            seq_id,
                            command: CMD_NOOP.to_owned(),
                            args,
                        })
                        .unwrap(),
                    );
                    crate::debug::PORT
                        .0
                        .send(ProtocolMessage {
                            ty: TYPE_RESPONSE.to_owned(),
                            others,
                        })
                        .await
                        .ok();
                    flow_rs::rt::task::sleep(std::time::Duration::from_secs(
                        (1f32 * div_ratio) as u64,
                    ))
                    .await;
                } else {
                    flow_rs::rt::task::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        })
    }
}
