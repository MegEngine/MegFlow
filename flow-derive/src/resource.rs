use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::Token;

pub struct ResourceDefine {
    name: syn::Expr,
    ty: syn::Type,
}
impl Parse for ResourceDefine {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: syn::Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty: syn::Type = input.parse()?;
        Ok(ResourceDefine { name, ty })
    }
}

pub fn registry_expand(input: ResourceDefine) -> TokenStream {
    let name = &input.name;
    let ty = &input.ty;
    quote! {
        flow_rs::submit!(#name.to_owned(),
            flow_rs::node::ResourceSlice{
                cons: Box::new(|name: String, args: &toml::value::Table| Box::new(<#ty>::new(name, args))),
            }
        );
    }
}
