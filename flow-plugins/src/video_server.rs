/**
 * \file flow-plugins/src/video_server.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use super::utils::error;
use ::log::error;
use anyhow::Result;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video;
use flow_rs::prelude::*;
use flow_rs::rt::sync::Mutex;
use futures_util::{pin_mut, select, stream::FuturesUnordered, FutureExt, StreamExt};
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use rweb::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Once;
use toml::value::Table;

type RwebResult = Result<String, Rejection>;

#[inputs(inp:dyn)]
#[outputs(out:dyn)]
#[derive(Node, Actor, Default)]
struct VideoServer {
    port: u16,
    resources: Option<ResourceCollection>,
}

#[derive(Serialize)]
struct VideoDescp {
    id: u64,
    url: String,
}

#[derive(Clone)]
struct State {
    mapping: Arc<Mutex<HashMap<u64, (VideoDescp, Sender)>>>,
    sender: flow_rs::rt::channel::Sender<(u64, String, oneshot::Sender<RwebResult>)>,
    counter: Arc<AtomicU64>,
}

struct Messages {
    mapping: HashMap<u64, Vec<String>>,
}

type MessagesWithLock = Arc<Mutex<Messages>>;

#[post("/start/{url}")]
#[openapi(summary = "start a video stream")]
async fn start(#[data] state: State, url: String) -> RwebResult {
    let id = state.id();
    let (s, r) = oneshot::channel();
    state.sender.send((id, url, s)).await.ok();
    r.await.unwrap()
}

#[post("/stop/{id}")]
#[openapi(summary = "stop a video stream by id")]
async fn stop(#[data] state: State, id: u64) -> RwebResult {
    if let Some((_, sender)) = state.mapping.lock().await.remove(&id) {
        sender.close();
        Ok(format!("stop stream {} success", id))
    } else {
        Err(reject::not_found())
    }
}

#[get("/list")]
#[openapi(summary = "list all video streams")]
async fn list(#[data] state: State) -> Result<impl Reply, Rejection> {
    let mapping = state.mapping.lock().await;
    let mut videos = vec![];
    for (descp, _) in mapping.values() {
        videos.push(descp);
    }
    Ok(rweb::reply::json(&videos))
}

#[get("/get_msgs/{id}")]
#[openapi(summary = "get messages from video analyze by id")]
async fn get_msgs(#[data] messages: MessagesWithLock, id: u64) -> Result<impl Reply, Rejection> {
    let mut messages = messages.lock().await;
    if let Some(messages) = messages.mapping.get_mut(&id) {
        let ret = Ok(rweb::reply::json(messages));
        messages.clear();
        ret
    } else {
        Err(reject::not_found())
    }
}

impl VideoServer {
    fn new(_: String, args: &Table) -> VideoServer {
        VideoServer {
            port: args["port"]
                .as_integer()
                .expect("expect args[port] in node[VideoServer]")
                .to_owned() as u16,
            ..Default::default()
        }
    }

    async fn initialize(&mut self, resources: ResourceCollection) {
        self.resources = Some(resources);
    }
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &flow_rs::graph::Context) -> Result<()> {
        let (s, r) = flow_rs::rt::channel::unbounded();
        let state = State::new(s);
        let mapping = state.mapping.clone();
        let messages = Messages::new().with_lock();
        let messages_cloned = messages.clone();

        let (spec, filter) = openapi::spec().build(move || {
            start(state.clone())
                .or(stop(state.clone()))
                .or(list(state))
                .or(get_msgs(messages_cloned))
        });

        let mut recv_msgs = FuturesUnordered::new();
        let mut recv_conns = FuturesUnordered::new();
        let mut spawn_decode = FuturesUnordered::new();
        let listen = serve(filter.or(openapi_docs(spec)))
            .run(([0, 0, 0, 0], self.port))
            .fuse();
        recv_conns.push(self.inp.fetch());
        spawn_decode.push(r.recv());

        pin_mut!(listen);

        loop {
            select! {
                // server listen
                _ = listen => break,
                // spawn task to wait message from subgraph
                conns = recv_conns.select_next_some() => {
                    if let Ok((id, port)) = conns {
                        let messages = messages.clone();
                        recv_msgs.push(async move {
                            while let Ok(mut msg) = port.recv().await {
                                let messages_map = &mut messages.lock().await.mapping;
                                let messages = messages_map.entry(id).or_default();
                                let msg = Python::with_gil(|py| -> PyResult<_> {
                                    let msg: PyObject = msg.unpack();
                                    let msg = msg.as_ref(py).extract()?;
                                    Ok(msg)
                                }).expect("plugin[VideoServier] only support python string as input");
                                messages.push(msg);
                            }
                        });
                        recv_conns.push(self.inp.fetch());
                    }
                },
                // wait message from subgraph
                _ = recv_msgs.select_next_some() => {},
                // spawn subgraph
                ret = spawn_decode.select_next_some() => {
                    if let Ok((id, url, waker)) = ret {
                        self.out.create(id, self.resources.clone().unwrap()).await.expect("broker has closed");
                        let (_, port) = self.out.fetch().await.expect("broker has closed");
                        let url = urlencoding::decode(&url).unwrap().into_owned();
                        let url_cloned = url.clone();
                        let port_cloned = port.clone();
                        let handle = flow_rs::rt::task::spawn_blocking(move || -> Result<(), ffmpeg_next::Error> {
                            if let Err(err) = decode_video(id, &url_cloned, &port_cloned) {
                                error!("video[{}] {} decode fault: {:?}", id, url_cloned, err);
                                Err(err)
                            } else {
                                port_cloned.close();
                                Ok(())
                            }
                        });
                        match flow_rs::rt::future::timeout(std::time::Duration::from_millis(200), handle).await {
                            Ok(ret) => match ret {
                                Ok(_) => {
                                    mapping.lock().await.insert(id, (VideoDescp { id, url }, port));
                                    waker.send(Ok(format!("start stream whose id is {}", id))).ok()
                                }
                                Err(err) => waker.send(Err(error::reject_cause(err))).ok(),
                            },
                            Err(_) => {
                                mapping.lock().await.insert(id, (VideoDescp { id, url }, port));
                                waker.send(Ok(format!("start stream whose id is {}", id))).ok()
                            }
                        };
                        spawn_decode.push(r.recv());
                    }
                },
            }
        }

        Ok(())
    }
}

node_register!("VideoServer", VideoServer);

impl State {
    fn new(
        sender: flow_rs::rt::channel::Sender<(u64, String, oneshot::Sender<RwebResult>)>,
    ) -> State {
        State {
            mapping: Default::default(),
            sender,
            counter: Arc::new(AtomicU64::new(0)),
        }
    }
    fn id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }
}

impl Messages {
    fn new() -> Messages {
        Messages {
            mapping: Default::default(),
        }
    }
    fn with_lock(self) -> MessagesWithLock {
        Arc::new(Mutex::new(self))
    }
}

static ONCE_INIT: Once = Once::new();

fn decode_video(
    id: u64,
    path: impl AsRef<Path>,
    sender: &Sender,
) -> Result<(), ffmpeg_next::Error> {
    ONCE_INIT.call_once(|| {
        ffmpeg_next::init().unwrap();
    });

    let mut ictx = input(&path)?;

    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg_next::Error::StreamNotFound)?;

    let video_stream_index = input.index();

    let mut decoder = input.codec().decoder().video()?;

    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::BGR24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let mut fid = 0;

    'main: for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            let mut decoded = Video::empty();

            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut bgr_frame = Video::empty();
                scaler.run(&decoded, &mut bgr_frame)?;

                let ndarray = Python::with_gil(|py| -> PyResult<_> {
                    let data = bgr_frame.data(0);
                    let ndarray = data.to_pyarray(py).reshape([
                        bgr_frame.height() as usize,
                        bgr_frame.stride(0) / 3,
                        3,
                    ])?;

                    Ok([("data", ndarray.to_object(py))]
                        .into_py_dict(py)
                        .to_object(py))
                })
                .unwrap();

                let mut envelope = Envelope::new(ndarray);
                envelope.info_mut().from_addr = Some(id);
                envelope.info_mut().partial_id = Some(fid);
                let ret = flow_rs::rt::task::block_on(async { sender.send(envelope).await });
                fid += 1;

                if matches!(ret, Err(_)) {
                    break 'main;
                }
            }
        }
    }

    decoder.send_eof()?;

    let mut decoded = Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {}

    Ok(())
}
