/**
 * \file flow-derive/src/ports.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::{
    lit::{self, string},
    type_name,
};
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    LitStr, Token,
};

enum PortType {
    Unit,
    List,
    Dict,
    Dyn,
}

struct Port {
    name: syn::Ident,
    ty: PortType,
}

impl Parse for Port {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: syn::Ident = input.parse()?;
        Ok(if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            if input.peek(syn::token::Dyn) {
                input.parse::<syn::token::Dyn>()?;
                Port {
                    name,
                    ty: PortType::Dyn,
                }
            } else if input.peek(syn::token::Bracket) {
                let _content;
                bracketed!(_content in input);
                Port {
                    name,
                    ty: PortType::List,
                }
            } else if input.peek(syn::token::Brace) {
                let _content;
                braced!(_content in input);
                Port {
                    name,
                    ty: PortType::Dict,
                }
            } else {
                return Err(input.error("expect []"));
            }
        } else {
            Port {
                name,
                ty: PortType::Unit,
            }
        })
    }
}

pub struct PortSequence {
    fields: Punctuated<Port, Token![,]>,
}

impl Parse for PortSequence {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(PortSequence {
            fields: input.parse_terminated(Port::parse).unwrap_or_default(),
        })
    }
}

pub fn in_expand(ports: &PortSequence) -> Vec<TokenStream> {
    expand(ports, type_name::IN_T)
}

pub fn out_expand(ports: &PortSequence) -> Vec<TokenStream> {
    expand(ports, type_name::OUT_T)
}

fn expand(ports: &PortSequence, extend_ty: &str) -> Vec<TokenStream> {
    let extend_ty = lit::ident(extend_ty);
    ports
        .fields
        .iter()
        .map(|port| {
            let Port { name, ty } = port;
            match ty {
                PortType::Unit => {
                    quote_spanned!(name.span()=> #name: flow_rs::channel::#extend_ty)
                }
                PortType::List => {
                    quote_spanned!(name.span()=> #name: Vec<flow_rs::channel::#extend_ty>)
                }
                PortType::Dict => {
                    quote_spanned!(name.span()=> #name: std::collections::HashMap<u64, flow_rs::channel::#extend_ty>)
                }
                PortType::Dyn => {
                    quote_spanned!(name.span()=> #name: flow_rs::node::DynPorts<flow_rs::channel::#extend_ty>)
                }
            }
        })
        .collect()
}

pub fn name_expand(ports: &PortSequence) -> Vec<LitStr> {
    ports
        .fields
        .iter()
        .map(|port| match port.ty {
            PortType::Dyn => string(format!("dyn@{}", port.name)),
            PortType::List => string(format!("[{}]", port.name)),
            PortType::Dict => string(format!("{{{}}}", port.name)),
            PortType::Unit => string(&port.name),
        })
        .collect()
}
