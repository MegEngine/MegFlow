/**
 * \file flow-plugins/src/utils/image.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use flow_rs::rt::io::prelude::WriteExt;
use futures_util::{pin_mut, StreamExt};
use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use hyper::Body;
use image::GenericImageView;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::io::Cursor;
use std::ops::Deref;
use std::str::FromStr;

use super::error::reject_cause;

use mime::Mime;
use rweb::openapi::{
    Entity, MediaType, ObjectOrReference, Response, ResponseEntity, Responses, Schema, Type,
};
use rweb::*;

pub struct Image(image::DynamicImage);

impl Image {
    pub fn into_bgr8(self) -> image::ImageBuffer<image::Bgr<u8>, Vec<u8>> {
        self.0.into_bgr8()
    }

    pub fn from_raw(w: u32, h: u32, buf: &[u8]) -> Option<Image> {
        let mut owned = vec![];
        owned.extend_from_slice(buf);
        image::ImageBuffer::from_raw(w, h, owned)
            .map(|buf| Image(image::DynamicImage::ImageBgr8(buf)))
    }
}

impl Entity for Image {
    fn describe() -> Schema {
        Schema {
            description: Cow::Borrowed("image"),
            schema_type: Some(Type::String),
            format: Cow::Borrowed("binary"),
            ..Default::default()
        }
    }
}

impl ResponseEntity for Image {
    fn describe_responses() -> Responses {
        let mut resps = IndexMap::new();

        let mut content = IndexMap::new();
        content.insert(
            Cow::Borrowed("image/jpeg"),
            MediaType {
                schema: Some(ObjectOrReference::Object(Self::describe())),
                examples: None,
                encoding: Default::default(),
            },
        );

        resps.insert(
            Cow::Borrowed("200"),
            Response {
                content,
                ..Default::default()
            },
        );
        resps
    }
}

impl Deref for Image {
    type Target = image::DynamicImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Image {
    type Filter = rweb::filters::BoxedFilter<(Image,)>;

    fn is_body() -> bool {
        true
    }

    fn content_type() -> &'static str {
        "image/*"
    }

    fn new() -> Self::Filter {
        rweb::header::<Mime>("content-type")
            .and(filters::body::stream())
            .and_then(extract_image)
            .boxed()
    }
}

impl Reply for Image {
    #[inline]
    fn into_response(self) -> reply::Response {
        use image::codecs::jpeg::JpegEncoder as Encoder;
        let mut buf = vec![];
        let mut encoder = Encoder::new(&mut buf);
        encoder
            .encode(self.as_bytes(), self.width(), self.height(), self.color())
            .unwrap();
        let len = buf.len();
        let mut resp = reply::Response::new(Body::from(buf));

        resp.headers_mut().typed_insert(ContentLength(len as u64));
        resp.headers_mut()
            .typed_insert(ContentType::from(Mime::from_str("image/jpeg").unwrap()));
        resp.headers_mut().typed_insert(AcceptRanges::bytes());

        resp
    }
}

async fn extract_image(
    mime: Mime,
    body: impl Stream<Item = Result<impl Buf, rweb::Error>>,
) -> Result<Image, Rejection> {
    if mime == mime::IMAGE_STAR {
        let mut buf = Vec::with_capacity(1024 * 1024 * 3);
        pin_mut!(body);
        while let Some(inner) = body.next().await {
            let inner = inner.map_err(reject_cause)?;
            buf.write_all(inner.chunk()).await.map_err(reject_cause)?;
        }

        use image::io::Reader as ImageReader;
        let reader = ImageReader::new(Cursor::new(buf))
            .with_guessed_format()
            .map_err(reject_cause)?;
        let image = Image(reader.decode().map_err(reject_cause)?);
        Ok(image)
    } else {
        Err(rweb::reject::reject())
    }
}
