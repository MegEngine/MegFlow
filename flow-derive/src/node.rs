/**
 * \file flow-derive/src/node.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::type_name;
use crate::utils::*;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{DeriveInput, Token};
pub struct NodeDefine {
    name: syn::Expr,
    ty: syn::Type,
}
impl Parse for NodeDefine {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: syn::Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty: syn::Type = input.parse()?;
        Ok(NodeDefine { name, ty })
    }
}
pub fn registry_expand(input: NodeDefine) -> TokenStream {
    let name = &input.name;
    let ty = &input.ty;
    quote! {
        flow_rs::submit!(#name.to_owned(),
            flow_rs::node::NodeSlice{
                cons: Box::new(|name: String, args: &toml::value::Table| Box::new(<#ty>::new(name, args))),
                info: flow_rs::node::NodeInfo {
                    inputs: <#ty>::inputs_name(),
                    outputs: <#ty>::outputs_name(),
                }
            }
        );
    }
}

pub fn expand(input: DeriveInput) -> TokenStream {
    let ident = input.ident;
    type IterArgs<'a> = ExtractParams<'a>;
    fn set_f((port_func, ident, ty): IterArgs) -> TokenStream {
        if match_last_ty(ty, "Vec") {
            quote_spanned! {ident.span()=>
                if port_name == concat!('[', stringify!(#ident), ']') {
                    if tag.is_none() {
                        if let Some(p) = self.#ident.iter_mut().find(|p| p.is_none()) {
                            *p = channel.#port_func();
                        } else {
                            self.#ident.push(channel.#port_func());
                        }
                    } else {
                        let i = tag.unwrap() as usize;
                        match self.#ident.len().cmp(&i) {
                            std::cmp::Ordering::Greater => {
                                debug_assert!(self.#ident[i].is_none());
                                self.#ident[i] = channel.#port_func();
                            }
                            std::cmp::Ordering::Less => {
                                self.#ident.resize(i+1, Default::default());
                                self.#ident[i] = channel.#port_func();
                            }
                            std::cmp::Ordering::Equal => {
                                self.#ident.push(channel.#port_func());
                            }
                        }
                    }
                } else
            }
        } else if match_last_ty(ty, "DynPorts") {
            quote! {}
        } else {
            quote_spanned! {ident.span()=>
                if port_name == stringify!(#ident) {
                    self.#ident = channel.#port_func();
                } else
            }
        }
    }
    fn set_dyn_f((_, ident, ty): IterArgs) -> TokenStream {
        if match_last_ty(ty, "DynPorts") {
            quote_spanned! {ident.span()=>
                if port_name == concat!("dyn@", stringify!(#ident)) {
                    self.#ident = flow_rs::node::DynPorts::new(local_key, target, cap, brokers);
                } else
            }
        } else {
            quote! {}
        }
    }
    fn close_f((_, ident, ty): IterArgs) -> TokenStream {
        if match_last_ty(ty, "Vec") {
            quote_spanned! {ident.span()=>
                self.#ident.clear();
            }
        } else {
            quote!()
        }
    }
    fn is_close_f((_, ident, ty): IterArgs) -> TokenStream {
        if match_last_ty(ty, "Vec") {
            quote_spanned! {ident.span()=>
                for chan in &self.#ident {
                    is_closed = is_closed && chan.is_closed();
                }
            }
        } else {
            quote_spanned! (ident.span()=> is_closed = is_closed && self.#ident.is_closed();)
        }
    }

    let seti = extract_ports(&input.data, type_name::IN_T, set_f);
    let seto = extract_ports(&input.data, type_name::OUT_T, set_f);
    let set_dyni = extract_ports(&input.data, type_name::IN_T, set_dyn_f);
    let set_dyno = extract_ports(&input.data, type_name::OUT_T, set_dyn_f);
    let closeo = extract_ports(&input.data, type_name::OUT_T, close_f);
    let is_closei = extract_ports(&input.data, type_name::IN_T, is_close_f);

    let (imp_g, ty_g, where_g) = input.generics.split_for_impl();
    quote! {
        impl#imp_g flow_rs::node::Node for #ident#ty_g
            #where_g {
                fn set_port(&mut self, port_name: &str, tag: Option<u64>, channel: &flow_rs::channel::ChannelStorage) {
                    #(#seti) *
                    #(#seto) *
                    { unreachable!() }
                }
                fn set_port_dynamic(
                    &mut self,
                    local_key: u64,
                    port_name: &str,
                    target: String,
                    cap: usize,
                    brokers: Vec<flow_rs::broker::BrokerClient>,
                ) {
                    #(#set_dyni) *
                    #(#set_dyno) *
                    { unreachable!() }
                }
                fn close(&mut self) {
                    #(#closeo)*
                }

                fn is_allinp_closed(&self) -> bool {
                    use flow_rs::channel::ChannelBase;
                    let mut is_closed = true;
                    #(#is_closei)*
                    is_closed
                }
            }
    }
}
