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
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
    DeriveInput, Field, Ident, Token, VisPublic, Visibility,
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
                    #name.notify();
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
                        #name.disable();
                    }
                }),
                disable: Box::new(|| {
                    #name.disable();
                }),
            });
        }
    }
}

pub fn parser_expand(input: DeriveInput) -> TokenStream {
    let ident = input.ident;
    let support_include = fields(&input.data).any(|field| {
        field
            .ident
            .as_ref()
            .map(|x| x == "include")
            .unwrap_or(false)
    });
    if support_include {
        let list: Vec<_> = fields(&input.data)
            .filter(|field| {
                match_last_ty(&field.ty, "Vec")
                    || match_last_ty(&field.ty, "VecDeque")
                    || match_last_ty(&field.ty, "LinkedList")
            })
            .map(|field| {
                let ident = &field.ident;
                quote_spanned! { ident.span()=>
                    config.#ident.append(&mut sub_config.#ident);
                }
            })
            .collect();
        let set: Vec<_> = fields(&input.data)
            .filter(|field| {
                match_last_ty(&field.ty, "HashSet") || match_last_ty(&field.ty, "BTreeSet")
            })
            .map(|field| {
                let ident = &field.ident;
                quote_spanned! {ident.span()=>
                    for x in std::mem::take(&mut sub_config.#ident) {
                        config.#ident.insert(x);
                    }
                }
            })
            .collect();
        let map: Vec<_> = fields(&input.data)
            .filter(|field| {
                match_last_ty(&field.ty, "HashMap") || match_last_ty(&field.ty, "BTreeMap")
            })
            .map(|field| {
                let ident = &field.ident;
                quote_spanned! {ident.span()=>
                    for (k, v) in std::mem::take(&mut sub_config.#ident) {
                        config.#ident.entry(k).or_insert(v);
                    }
                }
            })
            .collect();
        quote! {
            type PathSet = std::collections::BTreeSet<std::path::PathBuf>;
            impl<'a> #ident {
                fn from_impl(path: &std::path::Path, template: &str, dynamic: &templar::InnerData, mut visit: PathSet) -> anyhow::Result<(#ident, PathSet)> {
                    use templar::Context;

                    let template = flow_rs::config::parser::TEMPLAR.parse_template(template)?;
                    let context = templar::StandardContext::new();
                    context.set(dynamic.clone())?;
                    let content = template.render(&context)?;
                    let mut config: #ident = toml::from_str(&content)?;

                    let include: Vec<_> = config.include.iter().cloned().collect();

                    for clid_path in include {
                        let abs_path;
                        if clid_path.is_relative() {
                            let mut tmp = path.to_path_buf();
                            tmp.pop();
                            tmp.push(clid_path);
                            abs_path = tmp;
                        } else {
                            abs_path = clid_path;
                        }
                        if visit.contains(&abs_path) {
                            return Err(anyhow::anyhow!("cyclic include"));
                        }
                        let sub_template = std::fs::read_to_string(&abs_path)?;
                        visit.insert(abs_path.clone());
                        let mut ret = Self::from_impl(&abs_path, &sub_template, dynamic, visit)?;
                        let sub_config = &mut ret.0;
                        #(#list) *
                        #(#set) *
                        #(#map) *
                        visit = ret.1;
                    }
                    Ok((config, visit))
                }
            }
            impl<'a> flow_rs::config::parser::Parser<'a> for #ident {
                fn from_file(template: &std::path::Path, dynamic: Option<&std::path::Path>) -> anyhow::Result<#ident> {
                    let mut set = std::collections::BTreeSet::new();
                    set.insert(template.to_owned());

                    let dynamic = if let Some(dynamic) = dynamic {
                        std::fs::read_to_string(dynamic)?
                    } else {
                        "".to_owned()
                    };
                    let dynamic_data: templar::InnerData = toml::from_str(&dynamic)?;
                    let template_content = std::fs::read_to_string(template)?;

                    Self::from_impl(template, &template_content, &dynamic_data, set).map(|x| x.0)
                }

                fn from_str(template: &str, dynamic: Option<&str>) -> anyhow::Result<#ident> {
                    let dynamic_data: templar::InnerData = toml::from_str(dynamic.unwrap_or(""))?;
                    let cur = std::env::current_dir()?;
                    Self::from_impl(&cur, template, &dynamic_data, std::collections::BTreeSet::new()).map(|x| x.0)
                }
            }
        }
    } else {
        quote! {
            impl<'a> flow_rs::config::parser::Parser<'a> for #ident {
                fn from_file(template: &std::path::Path, dynamic: Option<&std::path::Path>) -> anyhow::Result<Self> {
                    let dynamic = if let Some(dynamic) = dynamic {
                        Some(std::fs::read_to_string(dynamic)?)
                    } else {
                        None
                    };
                    let template = std::fs::read_to_string(template)?;
                    Self::from_str(&template, dynamic.as_ref().map(|x| x.as_ref()))
                }

                fn from_str(template: &str, dynamic: Option<&str>) -> anyhow::Result<Self> {
                    use templar::Context;
                    use flow_rs::config::parser::TEMPLAR;
                    let dynamic_data: templar::InnerData = toml::from_str(dynamic.unwrap_or(""))?;
                    let template = TEMPLAR.parse_template(&template)?;
                    let context = templar::StandardContext::new();
                    context.set(dynamic_data)?;
                    let content = template.render(&context)?;
                    let config = toml::from_str(&content)?;
                    Ok(config)
                }
            }
        }
    }
}
