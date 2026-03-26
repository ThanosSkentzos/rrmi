use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

use crate::{
    RemoteObjectInfo,
    utils::{is_str_ref, normalize_type},
};

pub fn gen_listen(_remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    quote! {}
}

pub fn gen_handle_connection(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let struct_name = &remote_obj.struct_name.0;
    let req_name = Ident::new(&format!("{struct_name}Request"), Span::call_site());
    let res_name = Ident::new(&format!("{struct_name}Response"), Span::call_site());
    quote! {
        fn handle_connection(&self, mut stream: ::std::net::TcpStream) -> ::rrmi::RMIResult<()> {
            let request_bytes = ::rrmi::receive_data(&mut stream);
            let request: #req_name = ::rrmi::unmarshal(&request_bytes)?;

            let response: #res_name = self.handle_request(request);

            let response_bytes = ::rrmi::marshal(&response)?;
            ::rrmi::send_data(response_bytes, &mut stream)
    }
    }
}

pub fn gen_handle_request(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let struct_name = remote_obj.struct_name.0.clone();
    let req_name = Ident::new(&format!("{struct_name}Request"), Span::call_site());
    let res_name = Ident::new(&format!("{struct_name}Response"), Span::call_site());

    let match_arms = remote_obj.methods.iter().map(|m| {
        let method_name = &m.name;
        let variant = m.get_name_fixed();
        let params = &m.params.0;
        let (pattern, call) = if params.is_empty() {
            (
                quote! { #req_name::#variant },
                quote! { self.#method_name() },
            )
        } else {
            let param_names = params.iter().map(|p| &p.0.0);
            let param_names_2 = params.iter().map(|p| {
                let name = &p.0.0;
                let ty = &p.0.1;
                if is_str_ref(ty) {
                    quote! {&#name}
                } else {
                    quote! {#name}
                }
            });
            (
                quote! {#req_name::#variant { #(#param_names),*}},
                quote! {self.#method_name(#(#param_names_2),*)},
            )
        };
        quote! { #pattern => #res_name::#variant(#call)}
    });

    quote! {
        fn handle_request(&self, req: #req_name) -> #res_name{
            match req{
                #(#match_arms),*
            }
        }
    }
}

pub fn gen_enums(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let struct_name = remote_obj.struct_name.0.clone();
    let req_name = Ident::new(&format!("{struct_name}Request"), Span::call_site());
    let res_name = Ident::new(&format!("{struct_name}Response"), Span::call_site());
    let req_variants = remote_obj
        .methods
        .iter()
        .map(|m| {
            let enum_variant = m.get_name_fixed();
            let params = &m.params.0;
            if params.is_empty() {
                quote! { #enum_variant} // like List,
            } else {
                let fields = params.iter().map(|param| {
                    let (field_name, field_type) = &param.0;
                    let norm_type = normalize_type(field_type);
                    quote! {#field_name: #norm_type }
                });
                quote! { #enum_variant { #(#fields),*}} // like Lookup{name:String},
            }
        })
        .collect::<Vec<_>>();

    let res_variants = remote_obj.methods.iter().map(|m| {
        let enum_variant = m.get_name_fixed();
        let ret = m.get_ret();
        // if already_rmi_result(&ret) {
        //     quote! { #enum_variant(#ret)}
        // } else {
        //     quote! { #enum_variant(::rrmi::RMIResult<#ret>)}
        // }
        quote! { #enum_variant(#ret)}
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
        quote! {
            #enums
        }
    } else {
        quote! {
            #import_rmiresult
            #enums
        }
    }
}
