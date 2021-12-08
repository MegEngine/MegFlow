/**
 * \file flow-derive/src/utils.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use crate::lit;
use anyhow::Result;
use proc_macro2::TokenStream;
use std::collections::hash_map;
use std::hash::Hasher;
use syn::{parse::Parser, Field, Fields, Ident, ItemStruct, Type};

pub fn hash(input: &TokenStream) -> u64 {
    let mut hasher = hash_map::DefaultHasher::new();
    hasher.write(input.to_string().as_bytes());
    hasher.finish()
}

pub fn fields(data: &syn::Data) -> syn::punctuated::Iter<syn::Field> {
    match data {
        syn::Data::Struct(ref item) => match item.fields {
            syn::Fields::Named(ref fields) => fields.named.iter(),
            syn::Fields::Unnamed(ref fields) => fields.unnamed.iter(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

pub fn attr<'a>(attrs: &'a [syn::Attribute], name: &str) -> Option<&'a syn::Attribute> {
    attrs.iter().find(|attr| {
        attr.path
            .segments
            .first()
            .map(|p| p.ident == name)
            .unwrap_or(false)
    })
}

pub fn match_last_ty(ty: &syn::Type, wrapper: &str) -> bool {
    match ty {
        syn::Type::Path(ref p) => p
            .path
            .segments
            .last()
            .map(|seg| seg.ident == wrapper)
            .unwrap_or(false),
        _ => false,
    }
}

pub fn last_inner_ty(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.is_empty() {
            return None;
        }

        if let syn::PathArguments::AngleBracketed(ref inner_ty) =
            p.path.segments.last().unwrap().arguments
        {
            if inner_ty.args.is_empty() {
                return None;
            }

            let inner_ty = inner_ty.args.last().unwrap();
            if let syn::GenericArgument::Type(ref t) = inner_ty {
                return Some(t);
            }
        }
    }
    None
}

pub fn add_field(item: &mut ItemStruct, field: TokenStream) -> Result<()> {
    match item.fields {
        Fields::Named(ref mut fields) => fields.named.push(Field::parse_named.parse2(field)?),
        Fields::Unnamed(ref mut fields) => fields.unnamed.push(Field::parse_unnamed.parse2(field)?),
        _ => unreachable!(),
    }
    Ok(())
}

pub type ExtractParams<'a> = (&'a Ident, &'a Option<Ident>, &'a Type);

pub fn extract_ports(
    data: &syn::Data,
    ty_name: &str,
    f: fn(ExtractParams) -> TokenStream,
) -> Vec<TokenStream> {
    let port_func = lit::ident(ty_name.to_lowercase());
    fields(data)
        .filter(|field| {
            match_last_ty(&field.ty, ty_name)
                || last_inner_ty(&field.ty)
                    .map(|inner_ty| match_last_ty(inner_ty, ty_name))
                    .unwrap_or(false)
        })
        .map(|field| (&port_func, &field.ident, &field.ty))
        .map(f)
        .collect()
}
