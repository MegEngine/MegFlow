/**
 * \file flow-plugins/src/video_input.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::utils::codec;
use ::log::error;
use anyhow::Result;
use flow_rs::prelude::*;
use futures_util::future::join_all;
use serde::Serialize;
use std::path::Path;
use toml::value::Table;

#[inputs(inp:dyn)]
#[outputs(out:dyn)]
#[derive(Node, Default)]
struct VideoInput {
    urls: Vec<String>,
    repeat: u32,
}

#[derive(Serialize)]
struct VideoDescp {
    id: u64,
    url: String,
}

impl VideoInput {
    fn new(_: String, args: &Table) -> VideoInput {
        VideoInput {
            urls: args["urls"]
                .as_array()
                .expect("expect string array for urls")
                .iter()
                .map(|n| n.as_str().unwrap().to_owned())
                .collect(),
            repeat: args["repeat"]
                .as_integer()
                .expect("expect args[repeat] in node[VideoInput]")
                .to_owned() as u32,
            ..Default::default()
        }
    }
}

node_register!("VideoInput", VideoInput);

impl Actor for VideoInput {
    fn start(
        mut self: Box<Self>,
        _: Context,
        resources: ResourceCollection,
    ) -> rt::task::JoinHandle<()> {
        rt::task::spawn(async move {
            let mut recv_conns = vec![];

            let mut id = 0u64;
            for _ in 0..self.repeat {
                for url in self.urls.iter() {
                    if Path::new(&url).is_file() {
                        id += 1;
                        // create multiple stream
                        self.out
                            .create(id.to_owned(), resources.clone())
                            .await
                            .expect("broker has closed");
                        let (_, port) = self.out.fetch().await.expect("broker has closed");
                        let url_cloned = url.clone();
                        let port_cloned = port.clone();
                        flow_rs::rt::task::spawn_blocking(
                            move || -> Result<(), ffmpeg_next::Error> {
                                if let Err(err) = codec::decode_video(id, &url_cloned, &port_cloned)
                                {
                                    error!("video[{}] {} decode fault: {:?}", id, url_cloned, err);
                                    Err(err)
                                } else {
                                    port_cloned.close();
                                    Ok(())
                                }
                            },
                        );

                        // recv streams
                        let (_, port) = self.inp.fetch().await.unwrap();
                        recv_conns.push(async move { while port.recv_any().await.is_ok() {} });
                    }
                }
            }

            join_all(recv_conns).await;
        })
    }
}
