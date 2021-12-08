/**
 * \file flow-cffi/graph.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::error::*;
use crate::Message;
use flow_rs::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Once;

pub struct Graph {
    container: MainGraph,
    handle: Option<rt::task::JoinHandle<()>>,
    inps: HashMap<String, Sender>,
    outs: HashMap<String, Receiver>,
}

impl Graph {
    pub fn load<P: AsRef<Path>>(
        option: Option<flow_rs::loader::LoaderConfig>,
        config: P,
    ) -> Result<Graph> {
        static ONCE_INIT: Once = Once::new();
        ONCE_INIT.call_once(|| {
            flow_plugins::export();
        });
        let container = load(option, config).map_err(load_err)?;
        let mut inps = HashMap::new();
        let mut outs = HashMap::new();

        for name in container.input_names() {
            let inp = container.input(name).unwrap();
            inps.insert(name.to_owned(), inp);
        }

        for name in container.output_names() {
            let out = container.output(name).unwrap();
            outs.insert(name.to_owned(), out);
        }

        Ok(Graph {
            container,
            handle: None,
            inps,
            outs,
        })
    }

    pub async fn send(&self, name: &str, message: Message) -> Result<()> {
        let inp = self.inps.get(name).ok_or_else(|| not_found(name))?;
        inp.send(Envelope::new(message))
            .await
            .map_err(|_| closed(name))
    }

    pub async fn recv(&self, name: &str) -> Result<Message> {
        let out = self.outs.get(name).ok_or_else(|| not_found(name))?;
        out.recv::<Message>()
            .await
            .map(|mut x| x.unpack())
            .map_err(|_| closed(name))
    }

    pub async fn close_and_wait(&mut self) -> Result<()> {
        for inp in self.inps.values() {
            inp.close();
        }
        let handle = self.handle.take().ok_or_else(no_running)?;
        handle.await;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        if self.handle.is_none() {
            self.handle = Some(self.container.start());
        }

        Ok(())
    }
}
