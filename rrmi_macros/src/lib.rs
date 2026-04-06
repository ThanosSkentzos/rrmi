mod generators;
mod structure;
mod utils;

use crate::{
    generators::{gen_enums, gen_handle_connection, gen_handle_request, gen_remote_obj, gen_stub},
    structure::RemoteObjectInfo,
};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn remote_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let remote_obj = parse_macro_input!(item as RemoteObjectInfo);
    let original = &remote_obj.original;
    let debug_msg = format!("{remote_obj:?}");
    let _err = syn::Error::new_spanned(&remote_obj.struct_name.0, debug_msg).to_compile_error();

    let struct_name = &remote_obj.struct_name.0;
    let enums = gen_enums(&remote_obj);
    let handle_connection = gen_handle_connection(&remote_obj);
    let handle_request = gen_handle_request(&remote_obj);
    // let listen = gen_listen(&remote_obj);
    let stub = gen_stub(&remote_obj);
    let impl_remote_obj = gen_remote_obj(&remote_obj);

    // To test registry separately:
    // if struct_name == "Registry" {
    //     return quote! {
    //         #original
    //         #impl_remote_obj
    //         impl #struct_name{
    //             #handle_connection
    //             #handle_request
    //         }
    //     }
    //     .into();
    // }

    let q = quote! {
    #original
    #enums
    #stub
    #impl_remote_obj
    const _: () = {
        // #_err
        impl #struct_name{
            #handle_connection
            #handle_request
        }
    };
    };
    if struct_name == "PA1" {
        eprintln!("{q}");
    }
    q.into()
}
