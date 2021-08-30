/**
 * \file flow-rs/src/debug/server.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::feature::FeatureCommand;
use super::protocol::*;
use crate::{registry::Collect, rt::channel::*};
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use warp::ws::{Message, WebSocket};
use warp::Filter;

lazy_static::lazy_static! {
    pub static ref PORT: (Sender<ProtocolMessage>, Receiver<ProtocolMessage>) = unbounded();
}

static CONNECTED: AtomicBool = AtomicBool::new(false);

async fn connected(ws: WebSocket, graph: String) -> Result<()> {
    let (mut tx, mut rx) = ws.split();

    while PORT.1.try_recv().is_ok() {}
    let init_event = serde_json::to_string(&serde_json::json!({
        "ty": TYPE_EVENT,
        "event": EVENT_INITIALIZED,
        "features": FeatureCommand::registry_global().keys(),
        "graph": graph
    }))?;
    tx.send(Message::text(init_event)).await?;

    flow_rs::rt::task::spawn(async move {
        while let Ok(msg) = PORT.1.recv().await {
            if msg.ty == "__CLOSED__" {
                tx.send(Message::text(
                    serde_json::to_string(
                        &serde_json::json!({ "ty": TYPE_EVENT, "event": EVENT_TERMINATED }),
                    )
                    .unwrap(),
                ))
                .await
                .ok();
                break;
            }
            if tx
                .send(Message::text(serde_json::to_string(&msg).unwrap()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = rx.next().await {
        if let Ok(s) = msg.to_str() {
            let protocol: ProtocolMessage = serde_json::from_str(s)?;
            match protocol.ty.as_str() {
                TYPE_REQUEST => {
                    let req: RequestMessage =
                        serde_json::from_value(serde_json::Value::Object(protocol.others))?;
                    dowith_request(req).await?;
                }
                TYPE_EVENT => {
                    let event: EventMessage =
                        serde_json::from_value(serde_json::Value::Object(protocol.others))?;
                    dowith_event(event).await?;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

async fn reset() {
    FeatureCommand::registry_global().for_each(|cmd| {
        (cmd.disable)();
    });
    PORT.0
        .send(ProtocolMessage {
            ty: "__CLOSED__".to_owned(),
            others: Default::default(),
        })
        .await
        .unwrap();
    CONNECTED.store(false, Ordering::SeqCst);
}

async fn dowith_event(_: EventMessage) -> Result<()> {
    unimplemented!()
}
async fn dowith_request(req: RequestMessage) -> Result<()> {
    match req.command.as_str() {
        CMD_START => {
            let command = FeatureCommand::registry_global()
                .get(&req.feature)
                .ok_or_else(|| anyhow!("feature[{}] not found", req.feature))?;
            (command.start)(req.seq_id, serde_json::Value::Object(req.args));
        }
        CMD_STOP => {
            let command = FeatureCommand::registry_global()
                .get(&req.feature)
                .ok_or_else(|| anyhow!("feature[{}] not found", req.feature))?;
            (command.stop)(req.seq_id);
        }
        _ => unreachable!(),
    }
    Ok(())
}

/// A server for debug
pub struct Server {
    port: u16,
    graph: String,
}

impl Server {
    pub fn new(graph: String, port: u16) -> Server {
        Server { graph, port }
    }
    /// listen on 0.0.0.0:`self.port`
    pub async fn listen(self) {
        let graph = self.graph.clone();
        let graph = warp::any().map(move || graph.clone());
        let routes = warp::path("debugger").and(warp::ws()).and(graph).map(
            |ws: warp::ws::Ws, graph: String| {
                ws.on_upgrade(move |websocket| async move {
                    if CONNECTED
                        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        connected(websocket, graph.clone()).await.ok();
                        reset().await;
                    }
                })
            },
        );
        warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;
    }
}
