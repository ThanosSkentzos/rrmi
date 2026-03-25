mod generators;
mod structure;
mod utils;

use crate::{
    generators::{gen_enums, gen_handle_connection, gen_handle_request, gen_listen},
    structure::RemoteObjectInfo,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;

#[proc_macro_attribute]
pub fn remote_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse::<ItemImpl>(item.clone())
        .expect("remote_object should be used at an impl block");
    let remote_obj =
        RemoteObjectInfo::try_from(&mut input).expect("remote_object: failed to parse");

    let debug_msg = format!("{remote_obj:?}");
    let _err = syn::Error::new_spanned(&remote_obj.struct_name.0, debug_msg).to_compile_error();

    let struct_name = &remote_obj.struct_name.0;
    let enums = gen_enums(&remote_obj);
    let handle_request = gen_handle_request(&remote_obj);
    let handle_connection = gen_handle_connection(&remote_obj);
    let listen = gen_listen(&remote_obj);
    quote! {
        // #err
        #input
        #enums
        impl #struct_name{
            #handle_request
            #handle_connection
            #listen
        }
    }
    .into()
}
