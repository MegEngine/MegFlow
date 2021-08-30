/**
 * \file flow-plugins/src/utils/bare_json.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use headers::{ContentType, HeaderMapExt};
use indexmap::IndexMap;
use mime::Mime;
use rweb::openapi::*;
use rweb::*;
use std::borrow::Cow;
use std::str::FromStr;

#[derive(Schema)]
pub(crate) struct BareJson {
    inner: String,
}

impl BareJson {
    pub fn new(inner: String) -> BareJson {
        BareJson { inner }
    }
}

impl ResponseEntity for BareJson {
    fn describe_responses() -> Responses {
        let mut resps = IndexMap::new();

        let mut content = IndexMap::new();
        content.insert(
            Cow::Borrowed("application/json"),
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

impl Reply for BareJson {
    #[inline]
    fn into_response(self) -> reply::Response {
        let mut resp = reply::Response::new(self.inner.into());
        resp.headers_mut().typed_insert(ContentType::from(
            Mime::from_str("application/json").unwrap(),
        ));
        resp
    }
}
