mod structure;
mod utils;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, ItemImpl, ReturnType};

use crate::{
    structure::RemoteObjectInfo,
    utils::{already_rmi_result, fix_case, normalize_type},
};

#[proc_macro_attribute]
pub fn remote_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse::<ItemImpl>(item.clone())
        .expect("remote_object should be used at an impl block");
    let remote_obj =
        RemoteObjectInfo::try_from(&mut input).expect("remote_object: failed to parse");

    let debug_msg = format!("{remote_obj:?}");
    let _err = syn::Error::new_spanned(&remote_obj.struct_name.0, debug_msg).to_compile_error();
    let enums = gen_enums(&remote_obj);
    quote! {
        // #err
        #input
        #enums
    }
    .into()
}

fn gen_enums(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let struct_name = remote_obj.struct_name.0.clone();
    let req_name = Ident::new(&format!("{struct_name}Request"), Span::call_site());
    let res_name = Ident::new(&format!("{struct_name}Response"), Span::call_site());
    let req_variants = remote_obj
        .methods
        .iter()
        .map(|m| {
            let enum_variant = Ident::new(&fix_case(&m.name.to_string()), m.name.span());
            let fields = m.params.0.iter().map(|p| {
                let (field_name, field_type) = &p.0;
                let field_type = normalize_type(field_type);
                quote! {#field_name: #field_type }
            });
            if m.params.0.is_empty() {
                quote! { #enum_variant } // like List,
            } else {
                quote! { #enum_variant { #(#fields),*}} // like Lookup(String)
            }
        })
        .collect::<Vec<_>>();
    let res_variants = remote_obj.methods.iter().map(|m| {
        let enum_variant = Ident::new(&fix_case(&m.name.to_string()), m.name.span());
        let ret = match &m.ret {
            ReturnType::Default => syn::parse_quote!(()),
            ReturnType::Type(_, ty) => *ty.clone(),
        };
        if already_rmi_result(&ret) {
            quote! { #enum_variant(#ret)}
        } else {
            quote! { #enum_variant(::rrmi::RMIResult<#ret>)}
        }
    });

    let import_rmiresult = quote! {use rrmi::RMIResult;};
    let enums = quote! {
        #[derive(serde::Serialize,serde::Deserialize)]
        pub enum #req_name{
            #(#req_variants),*
        }

        #[derive(serde::Serialize,serde::Deserialize)]
        pub enum #res_name{
            #(#res_variants),*
        }
    };
    if struct_name == "Registry" {
        let _err = syn::Error::new_spanned(
            struct_name,
            "Registry is used internallyin rrmi, please use another name.",
        )
        .to_compile_error();
        quote! {
            // #_err
            #enums
        }
    } else {
        quote! {
            #import_rmiresult
            #enums
        }
    }
}
