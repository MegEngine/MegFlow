/**
 * \file flow-derive/src/actor.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::utils::*;
use crate::{lit, type_name};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, DeriveInput};

pub fn expand(input: DeriveInput) -> TokenStream {
    let ident = input.ident;
    let is_local = attr(&input.attrs, "local").is_some();
    let spawn_func = if is_local {
        lit::ident("spawn_local")
    } else {
        lit::ident("spawn")
    };

    fn send_empty_f((_, ident, ty): ExtractParams) -> TokenStream {
        if match_last_ty(ty, "Vec") {
            quote_spanned! {ident.span()=>
                for chan in &self.#ident {
                    chan.send_any(flow_rs::envelope::DummyEnvelope{}.seal()).await.ok();
                }
            }
        } else if match_last_ty(ty, "DynPorts") {
            quote_spanned! (ident.span()=>
                for chan in self.#ident.cache().values() {
                    chan.send_any(flow_rs::envelope::DummyEnvelope{}.seal()).await.ok();
                }
            )
        } else {
            quote_spanned! (ident.span()=>self.#ident.send_any(flow_rs::envelope::DummyEnvelope{}.seal()).await.ok();)
        }
    }
    fn min_empty_n_f((_, ident, ty): ExtractParams) -> TokenStream {
        if match_last_ty(ty, "Vec") {
            quote_spanned! {ident.span()=>
                for chan in &self.#ident {
                    min_empty_n = std::cmp::min(chan.empty_n(), min_empty_n)
                }
            }
        } else if match_last_ty(ty, "DynPorts") {
            quote_spanned! (ident.span()=>
                for chan in self.#ident.cache().values() {
                    min_empty_n = std::cmp::min(chan.empty_n(), min_empty_n)
                }
            )
        } else {
            quote_spanned! (ident.span()=> min_empty_n = std::cmp::min(self.#ident.empty_n(), min_empty_n);)
        }
    }

    let send_empty = extract_ports(&input.data, type_name::OUT_T, send_empty_f);
    let recv_empty_n = extract_ports(&input.data, type_name::IN_T, min_empty_n_f);
    let inputs_n = recv_empty_n.len();

    let (imp_g, ty_g, where_g) = input.generics.split_for_impl();
    quote! {
        impl#imp_g flow_rs::node::Actor for #ident#ty_g
            #where_g {
                fn start(mut self: Box<Self>, ctx: flow_rs::graph::Context, resources: flow_rs::resource::ResourceCollection) -> flow_rs::rt::task::JoinHandle<()> {
                    flow_rs::rt::task::#spawn_func(async move {

                        self.initialize(resources).await;
                        let mut empty_n = 0;
                        loop  {
                            let ret = self.exec(&ctx).await;
                            if #inputs_n > 0 {
                                let mut min_empty_n = usize::MAX;
                                #(#recv_empty_n )*

                                for _ in empty_n..min_empty_n {
                                    #(#send_empty )*
                                }

                                empty_n = min_empty_n;
                            }
                            match ret {
                                Ok(_) if self.is_allinp_closed() => break,
                                Err(_) => {
                                    ctx.close();
                                    break;
                                },
                                _ => (),
                            }
                        }
                        self.close();
                        self.finalize().await;
                    })
                }
            }
    }
}
