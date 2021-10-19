/**
 * \file flow-plugins/src/utils/codec.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video;
use flow_rs::prelude::*;
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::path::Path;
use std::sync::Once;

static ONCE_INIT: Once = Once::new();

pub fn decode_video(
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

                let envelope = Envelope::with_info(
                    ndarray,
                    EnvelopeInfo {
                        from_addr: Some(id),
                        partial_id: Some(fid),
                        ..Default::default()
                    },
                );
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
