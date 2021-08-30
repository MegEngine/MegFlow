/**
 * \file flow-derive/src/internal.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::{lit, utils::*};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
    Field, Ident, Token, VisPublic, Visibility,
};

pub struct CollectionSlice<ID> {
    name: ID,
    expr: TokenStream,
}

impl<ID: Parse> Parse for CollectionSlice<ID> {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: ID = input.parse()?;
        input.parse::<Token![,]>()?;
        let expr: TokenStream = input.parse()?;
        Ok(CollectionSlice { name, expr })
    }
}

pub fn submit_expand<ID: ToTokens>(input: CollectionSlice<ID>) -> TokenStream {
    let name = &input.name;
    let expr = &input.expr;
    let init = Ident::new(&format!("__init{}", hash(expr)), Span::call_site());
    quote! {
        #[allow(non_upper_case_globals)]
        #[flow_rs::ctor]
        fn #init() {
            flow_rs::registry::__submit_only_in_ctor(#name, #expr);
        }
    }
}

pub struct FeatureDeclare {
    name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for FeatureDeclare {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let content;
        braced!(content in input);
        let fields = content
            .parse_terminated(Field::parse_named)
            .unwrap_or_default();

        Ok(FeatureDeclare { name, fields })
    }
}

pub fn feature_expand(mut input: FeatureDeclare) -> TokenStream {
    let name = &input.name;
    let args_type = lit::ident(format!("__args{}", name));
    let args_name = lit::ident(format!("{}_args", name));
    let init = lit::ident(format!("__init{}", name));
    let fields = &mut input.fields;
    fields.iter_mut().for_each(|field| {
        field.vis = Visibility::Public(VisPublic {
            pub_token: Token![pub](field.span()),
        });
    });
    quote! {
        #[derive(serde::Deserialize, Default)]
        pub struct #args_type {
            #fields
        }
        pub static #name: feature::Feature = feature::Feature::new();
        lazy_static::lazy_static! {
            pub static ref #args_name: std::sync::RwLock<std::collections::HashMap<usize, #args_type>>
                = std::sync::RwLock::new(std::collections::HashMap::new());
        }
        #[flow_rs::ctor]
        fn #init() {
            flow_rs::registry::__submit_only_in_ctor(stringify!(#name), feature::FeatureCommand {
                start: Box::new(|k, json|  {
                    if #name.enable
                        .compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_ok() {
                       #name.notify();
                    }
                    let mut args = #args_name.write().unwrap();
                    args.insert(k, serde_json::from_value(json).unwrap());
                }),
                stop: Box::new(|k|  {
                    let mut args = #args_name.write().unwrap();
                    if let Some(_) = args.remove(&k) {
                        let into_object = |json| match json {
                            serde_json::Value::Object(json) => json,
                            _ => unreachable!(),
                        };

                        let args = into_object(serde_json::json!({
                            "seq_id": k,
                        }));

                        let others = into_object(
                            serde_json::to_value(protocol::EventMessage {
                                event: protocol::EVENT_STOP.to_owned(),
                                args,
                            })
                            .unwrap(),
                        );
                        server::PORT.0.try_send(protocol::ProtocolMessage {
                            ty: protocol::TYPE_EVENT.to_owned(),
                            others,
                        }).ok();
                    }
                    if args.is_empty() {
                        #name.enable.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                }),
                disable: Box::new(|| {
                    #name.enable.store(false, std::sync::atomic::Ordering::Relaxed);
                }),
            });
        }
    }
}
