/**
 * \file flow-plugins/src/image_server.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::utils::bare_json::BareJson;
use crate::utils::either::Either;
use crate::utils::error::reject_cause;
use crate::utils::image::Image;
use anyhow::Result;
use flow_rs::prelude::*;
use flow_rs::rt::sync::Mutex;
use futures_util::join;
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};
use rweb::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use toml::value::Table;

#[inputs(inp)]
#[outputs(out)]
#[derive(Node, Actor)]
struct ImageServer {
    port: u16,
    ty: RespTy,
}

type Mapping = HashMap<u64, oneshot::Sender<PyObject>>;

#[derive(Copy, Clone)]
enum RespTy {
    Image,
    Json,
}

#[derive(Clone)]
struct State {
    ty: RespTy,
    mapping: Arc<Mutex<Mapping>>,
    out: Sender,
    counter: Arc<AtomicU64>,
}

#[post("/analyze/{extra_data}")]
#[openapi(summary = "analyze an image")]
async fn analyze(
    #[data] state: State,
    img: Image,
    extra_data: String,
) -> Result<Either<Image, impl Reply>, Rejection> {
    let img = img.into_bgr8();
    let id = state.id();

    let pyobject: PyObject = Python::with_gil(|py| -> PyResult<_> {
        let data = img.as_raw();
        let ndarray =
            data.to_pyarray(py)
                .reshape([img.height() as usize, img.width() as usize, 3])?;

        Ok([
            ("data", ndarray.to_object(py)),
            ("extra_data", extra_data.to_object(py)),
        ]
        .into_py_dict(py)
        .into())
    })
    .map_err(reject_cause)?;

    let r = {
        let (s, r) = oneshot::channel();
        state.mapping.lock().await.insert(id, s);
        let envelope = Envelope::with_info(
            pyobject,
            EnvelopeInfo {
                partial_id: Some(id),
                ..Default::default()
            },
        );
        state.out.send(envelope).await.ok();
        r
    };
    let message = r.await.map_err(reject_cause)?;
    match state.ty {
        RespTy::Image => Python::with_gil(|py| -> PyResult<_> {
            let dict: &PyDict = message.extract(py)?;
            flow_rs::helper::uget_slice(py, dict.get_item("data").expect("error key<data>"))
        })
        .map_err(reject_cause)
        .map(|data| Image::from_raw(img.width(), img.height(), &data).unwrap())
        .map(Either::Left),
        RespTy::Json => Python::with_gil(|py| message.extract::<String>(py))
            .map_err(reject_cause)
            .map(BareJson::new)
            .map(Either::Right),
    }
}

impl ImageServer {
    fn new(_: String, args: &Table) -> ImageServer {
        let response = args.get("response").map(|resp| resp.as_str()).flatten();
        ImageServer {
            ty: if response == Some("json") {
                RespTy::Json
            } else {
                RespTy::Image
            },
            port: args["port"]
                .as_integer()
                .expect("expect args[port] in node[ImageServer]")
                .to_owned() as u16,
            inp: Default::default(),
            out: Default::default(),
        }
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        let state = State::new(self.ty, std::mem::take(&mut self.out));
        let mapping = state.mapping.clone();
        let inp = std::mem::take(&mut self.inp);

        let (spec, filter) = openapi::spec().build(move || analyze(state));

        join! {
            serve(filter.or(openapi_docs(spec))).run(([0, 0, 0, 0], self.port)),
            async move {
                while let Ok(mut msg) = inp.recv::<PyObject>().await {
                    let id = msg.info().partial_id.expect("partial_id required by image_server");
                    let msg = msg.unpack();
                    let mut mapping = mapping.lock().await;
                    if let Some(sender) = mapping.remove(&id) {
                        sender.send(msg).ok();
                    }
                }
            }
        };
        Ok(())
    }
}

node_register!("ImageServer", ImageServer);

impl State {
    fn new(ty: RespTy, out: Sender) -> State {
        State {
            ty,
            mapping: Default::default(),
            out,
            counter: Arc::new(AtomicU64::new(0)),
        }
    }
    fn id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }
}
