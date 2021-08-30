/**
 * \file flow-derive/src/lib.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
mod actor;
mod internal;
mod lit;
mod node;
mod ports;
mod resource;
mod type_name;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

/// A proc macro used to define inputs of a node. `#[inputs(port_name[:port_type], ..)]`
///
/// |  Annotation  |  Description |
/// | :-  | :- |
/// | `port_name` | SimplePort, which is a Receiver of a MPMC channel |
/// | `port_name: []` | ListPort, which is `Vec<SimplePort>` |
/// | `port_name: dyn` | DynamicPort, which is a port created dynamic, see `flow_rs::node::DynamicPort` for more detail |
#[proc_macro_attribute]
pub fn inputs(ports: TokenStream, input: TokenStream) -> TokenStream {
    let ports = parse_macro_input!(ports as ports::PortSequence);
    let fields = ports::in_expand(&ports);
    let names = ports::name_expand(&ports);
    let item_struct = parse_macro_input!(input as ItemStruct);
    ports_expand("inputs", item_struct, fields, names).into()
}

/// A proc macro used to define outputs of a node. `#[outputs(port_name[:port_type], ..)]`
///
/// |  Annotation  |  Description |
/// | :-  | :- |
/// | `port_name` | SimplePort, which is a Sender of a MPMC channel |
/// | `port_name: []` | ListPort, which is `Vec<SimplePort>` |
/// | `port_name: dyn` | DynamicPort, which is a port created dynamic, see `flow_rs::node::DynamicPort` for more detail |
#[proc_macro_attribute]
pub fn outputs(ports: TokenStream, input: TokenStream) -> TokenStream {
    let ports = parse_macro_input!(ports as ports::PortSequence);
    let fields = ports::out_expand(&ports);
    let names = ports::name_expand(&ports);
    let item_struct = parse_macro_input!(input as ItemStruct);
    ports_expand("outputs", item_struct, fields, names).into()
}

#[proc_macro_derive(Node)]
pub fn node_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    node::expand(input).into()
}

/// attributes(local): spawn actor by `flow_rs::rt::task::spawn_local`
#[proc_macro_derive(Actor, attributes(local))]
pub fn actor_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    actor::expand(input).into()
}

/// A proc macro used to register a node. `node_register!("NodeType", NodeType)`
#[proc_macro]
pub fn node_register(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as node::NodeDefine);
    node::registry_expand(input).into()
}

/// A proc macro used to register a resource. `resource_register!("ResourceType", ResourceType)`
#[proc_macro]
pub fn resource_register(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as resource::ResourceDefine);
    resource::registry_expand(input).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn submit(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as internal::CollectionSlice<syn::Expr>);
    internal::submit_expand(input).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn feature(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as internal::FeatureDeclare);
    internal::feature_expand(input).into()
}

fn ports_expand(
    prefix: &str,
    mut item_struct: ItemStruct,
    fields: Vec<proc_macro2::TokenStream>,
    names: Vec<syn::LitStr>,
) -> proc_macro2::TokenStream {
    for field in fields {
        utils::add_field(&mut item_struct, field).unwrap();
    }
    let ident = &item_struct.ident;
    let func_ident = lit::ident(format!("{}_name", prefix));
    let (imp_g, ty_g, where_g) = item_struct.generics.split_for_impl();
    quote! {
        #item_struct
        impl#imp_g #ident#ty_g #where_g {
            fn #func_ident() -> Vec<String> {
                vec![#(#names),*].into_iter().map(|n: &str| n.to_owned()).collect()
            }
        }
    }
}
