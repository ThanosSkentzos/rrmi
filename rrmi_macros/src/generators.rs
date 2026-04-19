use quote::quote;
use syn::Ident;

use crate::{
    RemoteObjectInfo, Span, TokenStream2,
    utils::{already_rmi_result, fix_ref_to_type, fix_ref_when_called, is_str_ref},
};

pub fn gen_remote_obj(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let struct_name = &remote_obj.struct_name.0;
    quote! {
        impl RemoteObject for #struct_name{

            #[cfg_attr(feature = "tracing", tracing::instrument)]
            fn run(&self, stream: &mut ::rrmi::TcpStream) -> ::rrmi::RMIResult<()> {
                self.handle_connection_gen(stream)
        }
            fn name(&self) -> &'static str{
                stringify!(#struct_name)
            }
    }
    }
}

pub fn gen_stub(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let struct_name = &remote_obj.struct_name.0;
    let (req_name, res_name) = remote_obj.get_enum_names();
    let stub_name = Ident::new(&format!("{struct_name}Stub"), Span::call_site());
    let functions = remote_obj.methods.iter().map(|m| {
        let method_name = m.name.clone();
        let camel = m.get_name_camel();
        let params = &m.params.0;
        let param_name_types = params.iter().map(|p| {
            let name = &p.0.0;
            let ty = &p.0.1;
            quote! { #name: #ty}
        }); // iterator over a:i32 , b:i32, c: &str

        let mut ret = m.get_ret();
        let mut pattern = quote! {#res_name::#camel(res)};
        let mut expr = quote! {Ok(res)};

        if struct_name == "Registry" {
            pattern = quote! {#res_name::#camel(Ok(res))};
            if method_name == "lookup" {
                expr = quote! {Ok(::rrmi::Stub::new(res))};
                ret = syn::parse_quote!(::rrmi::Stub);
            }
        }

        let ret = if already_rmi_result(&ret) {
            quote! {#ret}
        } else {
            quote! {::rrmi::RMIResult<#ret>}
        };
        let param_names = params.iter().map(|p| fix_ref_when_called(&p.0));

        let fn_contents = quote! {
            use ::rrmi::Transport;
            let transport_client = &self.transport_client;
            let req = #req_name::#camel{
                #(#param_names),*
            };
            let resp : #res_name = transport_client.send(req)?;
            match resp{
                #pattern => #expr,
                _ => Err(::rrmi::RMIError::TransportError("Wrong response".to_string())),
            }
        };

        let fn_call = if params.is_empty() {
            quote! {pub fn #method_name(&self) -> #ret{
                #fn_contents
            } }
        } else {
            quote! {pub fn #method_name(&self, #(#param_name_types),* ) -> #ret{
                #fn_contents
            } }
        };
        quote! {#fn_call}
    });

    let stub_struct = quote! {
        pub struct #stub_name{
            // remote: ::rrmi::RemoteRef,
            transport_client: ::rrmi::TcpClient,
            stub_name: String,
        }
        impl From<::rrmi::Stub> for #stub_name{
            fn from(stub: ::rrmi::Stub) -> Self{
                let remote = stub.remote;
                let transport_client = ::rrmi::TcpClient::new(remote.addr);
                #stub_name{transport_client, stub_name: "#stub_name".into()}
            }
        }
    };

    quote! {
        #[cfg_attr(feature = "tracing", derive(Debug))]
        #stub_struct
        impl #stub_name{
        #(#functions)*
        }
    }
}

pub fn gen_handle_connection(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let (req_name, res_name) = remote_obj.get_enum_names();
    quote! {
        #[cfg_attr(feature = "tracing", ::tracing::instrument)]
        fn handle_connection_gen(&self, stream: &mut ::rrmi::TcpStream) -> ::rrmi::RMIResult<()> {
            let request_bytes = ::rrmi::receive_data(stream);
            let request: #req_name = ::rrmi::unmarshal(&request_bytes)?;

            let response: #res_name = self.handle_request_gen(request);

            let response_bytes = ::rrmi::marshal(&response)?;
            ::rrmi::send_data(response_bytes, stream)
    }
    }
}

pub fn gen_handle_request(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let (req_name, res_name) = remote_obj.get_enum_names();
    let match_arms = remote_obj.methods.iter().map(|m| {
        let method_name = &m.name;
        let camel = m.get_name_camel();
        let params = &m.params.0;
        let (pattern, call) = if params.is_empty() {
            (quote! { #req_name::#camel }, quote! { self.#method_name() })
        } else {
            let param_names = params.iter().map(|p| &p.0.0);
            let param_names_with_ref = params.iter().map(|p| {
                let name = &p.0.0;
                let ty = &p.0.1;
                if is_str_ref(ty) {
                    quote! {&#name}
                } else {
                    quote! {#name}
                }
            });
            (
                quote! {#req_name::#camel { #(#param_names),*}},
                quote! {self.#method_name(#(#param_names_with_ref),*)},
            )
        };
        quote! { #pattern => #res_name::#camel(#call)}
    });
    quote! {
        #[cfg_attr(feature = "tracing", ::tracing::instrument)]
        fn handle_request_gen(&self, req: #req_name) -> #res_name{
            match req{
                #(#match_arms),*
            }
        }
    }
}

pub fn gen_enums(remote_obj: &RemoteObjectInfo) -> TokenStream2 {
    let (req_name, res_name) = remote_obj.get_enum_names();
    let req_variants = remote_obj
        .methods
        .iter()
        .map(|m| {
            let enum_variant = m.get_name_camel();
            let params = &m.params.0;
            if params.is_empty() {
                quote! { #enum_variant} // like List,
            } else {
                let fields = params.iter().map(|param| {
                    let (field_name, field_type) = &param.0;
                    let norm_type = fix_ref_to_type(field_type);
                    quote! {#field_name: #norm_type }
                });
                quote! { #enum_variant { #(#fields),*}} // like Lookup{name:String},
            }
        })
        .collect::<Vec<_>>();

    let res_variants = remote_obj.methods.iter().map(|m| {
        let enum_variant = m.get_name_camel();
        let ret = m.get_ret();
        // if already_rmi_result(&ret) {
        //     quote! { #enum_variant(#ret)}
        // } else {
        //     quote! { #enum_variant(::rrmi::RMIResult<#ret>)}
        // }
        quote! { #enum_variant(#ret)}
    });

    let enums = quote! {
        #[derive(serde::Serialize,serde::Deserialize)]
        #[cfg_attr(feature = "tracing", derive(Debug))]
        pub enum #req_name{
            #(#req_variants),*
        }

        #[derive(serde::Serialize,serde::Deserialize)]
        #[cfg_attr(feature = "tracing", derive(Debug))]
        pub enum #res_name{
            #(#res_variants),*
        }
    };

    // if remote_obj.struct_name.0 == "MockRemoteObject" {
    //     return quote! {#enums};
    // }
    quote! {
        #enums
    }
}
