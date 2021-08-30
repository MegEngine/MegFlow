/**
 * \file flow-derive/src/lit.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use proc_macro2::Span;
use std::fmt::Display;

use syn::{Ident, LitStr};

pub fn string<T: Display>(lit: T) -> LitStr {
    LitStr::new(lit.to_string().as_str(), Span::call_site())
}

pub fn ident(lit: impl AsRef<str>) -> Ident {
    Ident::new(lit.as_ref(), Span::call_site())
}
