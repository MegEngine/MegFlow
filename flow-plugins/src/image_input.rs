/**
 * \file flow-plugins/src/image_input.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use anyhow::Result;
use flow_rs::prelude::*;
use futures_util::join;
use image::io::Reader as ImageReader;
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::fs;
use std::path::Path;
use toml::value::Table;

#[inputs(inp)]
#[outputs(out)]
#[derive(Node, Actor)]
struct ImageInput {
    urls: Vec<String>,
}

impl ImageInput {
    fn new(_: String, args: &Table) -> ImageInput {
        ImageInput {
            urls: args["urls"]
                .as_array()
                .expect("expect string array for urls")
                .iter()
                .map(|n| n.as_str().unwrap().to_owned())
                .collect(),
            inp: Default::default(),
            out: Default::default(),
        }
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        let out = std::mem::take(&mut self.out);
        let urls = std::mem::take(&mut self.urls);
        let decode = rt::task::spawn(async move {
            let mut partial_id = 0u64;
            for path in &urls {
                if Path::new(&path).is_dir() {
                    match fs::read_dir(&path) {
                        Err(why) => {
                            println!("{:?}", why.kind());
                            break;
                        }
                        Ok(dirs) => {
                            for dir in dirs {
                                let filepath = &dir.unwrap().path();
                                if Path::new(&filepath).is_file() {
                                    // skip sub directory
                                    let decode_img = ImageReader::open(&filepath)
                                        .unwrap()
                                        .decode()
                                        .expect("decode image fail");
                                    let img = decode_img.into_bgr8();

                                    let pyobject: PyObject =
                                        Python::with_gil(|py| -> PyResult<_> {
                                            let data = img.as_raw();
                                            let ndarray = data.to_pyarray(py).reshape([
                                                img.height() as usize,
                                                img.width() as usize,
                                                3,
                                            ])?;

                                            Ok([
                                                ("data", ndarray.to_object(py)),
                                                (
                                                    "extra_data",
                                                    Path::new(&filepath).file_stem().to_object(py),
                                                ),
                                            ]
                                            .into_py_dict(py)
                                            .into())
                                        })
                                        .unwrap();

                                    let mut envelope = Envelope::new(pyobject);
                                    envelope.info_mut().partial_id = Some(partial_id);
                                    partial_id += 1;
                                    out.send(envelope).await.ok();
                                }
                            }
                        }
                    }
                }
            }
        });

        let inp = std::mem::take(&mut self.inp);
        let receiver =
            rt::task::spawn(async move { while inp.recv::<PyObject>().await.is_ok() {} });

        join!(receiver, decode);
        Ok(())
    }
}

node_register!("ImageInput", ImageInput);
