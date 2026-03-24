use std::fmt::Debug;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, ReturnType, Token, Type,
    punctuated::Punctuated,
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
    let name = remote_obj.struct_name.0.clone();
    let req_name = Ident::new(&format!("{name}Request"), Span::call_site());
    let res_name = Ident::new(&format!("{name}Response"), Span::call_site());
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
        quote! { #enum_variant(::rrmi::RMIResult<#ret>)}
    });

    quote! {
        use rrmi::RMIResult;
        // #[derive(serde::Serialize,serde::Deserialize)]
        pub enum #req_name{
            #(#req_variants),*
        }

        // #[derive(serde::Serialize,serde::Deserialize)]
        pub enum #res_name{
            #(#res_variants),*
        }
    }
}

struct RemoteObjectInfo {
    struct_name: StructNameInfo,
    methods: Vec<RemoteMethodInfo>,
}

struct StructNameInfo(Ident);

struct RemoteMethodInfo {
    name: Ident,
    params: ParametersInfo,
    ret: ReturnType,
}

struct ParametersInfo(Vec<ParameterInfo>);

struct ParameterInfo((Ident, Type));

impl TryFrom<&mut ItemImpl> for RemoteObjectInfo {
    type Error = ();
    fn try_from(impl_block: &mut ItemImpl) -> Result<Self, ()> {
        let struct_name = StructNameInfo::try_from(impl_block.self_ty.as_ref())?;
        let methods: Vec<RemoteMethodInfo> = impl_block
            .items
            .iter_mut()
            .filter_map(|x| {
                match x {
                    ImplItem::Fn(method) => RemoteMethodInfo::try_from(method),
                    _ => Err(()),
                }
                .ok()
            })
            .collect();
        Ok(Self {
            struct_name,
            methods,
        })
    }
}
impl Debug for RemoteObjectInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let methods = self
            .methods
            .iter()
            .map(|m| format!(" {:?}", m))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "struct {:?}\nmethods:\n{}", self.struct_name, methods)
    }
}

impl TryFrom<&Type> for StructNameInfo {
    type Error = ();
    fn try_from(struct_name: &Type) -> Result<Self, ()> {
        match struct_name {
            Type::Path(p) => Ok(Self(
                p.path
                    .segments
                    .last()
                    .expect("type should have at least one segment")
                    .ident
                    .clone(),
            )),
            _ => Err(()),
        }
    }
}

impl Debug for StructNameInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&mut ImplItemFn> for RemoteMethodInfo {
    type Error = ();
    fn try_from(method: &mut ImplItemFn) -> Result<Self, ()> {
        let is_remote = method
            .attrs
            .iter()
            .any(|attr| matches!(&attr.meta, Meta::Path(path) if path.is_ident("remote")));
        if !is_remote {
            return Err(());
        }
        // DISCARD #[remote]
        method
            .attrs
            .retain(|a| !matches!(&a.meta, Meta::Path(path) if path.is_ident("remote")));
        let name = method.sig.ident.clone();
        let params = ParametersInfo::from(&method.sig.inputs);
        let ret = method.sig.output.clone();
        Ok(Self { name, params, ret })
    }
}

impl Debug for RemoteMethodInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = &self.ret;
        write!(f, "fn {}{:?} {}", self.name, self.params, quote!(#ret))
    }
}

impl From<&Punctuated<FnArg, Token![,]>> for ParametersInfo {
    fn from(inputs: &Punctuated<FnArg, Token![,]>) -> Self {
        inputs
            .iter()
            .filter_map(|arg| ParameterInfo::try_from(arg).ok())
            .collect()
    }
}

impl FromIterator<ParameterInfo> for ParametersInfo {
    // need this to be able to collect
    fn from_iter<T: IntoIterator<Item = ParameterInfo>>(iter: T) -> Self {
        ParametersInfo(iter.into_iter().collect())
    }
}

impl Debug for ParametersInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self
            .0
            .iter()
            .map(|p| format!("{:?}", p))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "({})", params)
    }
}

impl TryFrom<&FnArg> for ParameterInfo {
    type Error = ();
    fn try_from(arg: &FnArg) -> Result<Self, ()> {
        match arg {
            FnArg::Receiver(_) => Err(()),
            FnArg::Typed(pt) => match pt.pat.as_ref() {
                Pat::Ident(pi) => Ok(Self((pi.ident.clone(), *pt.ty.clone()))),
                _ => Err(()),
            },
        }
    }
}

impl Debug for ParameterInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, ty) = &self.0;
        write!(f, "{name}:{}", quote!(#ty))
    }
}

fn fix_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

fn normalize_type(ty: &Type) -> Type {
    // check if the type is &str
    if let Type::Reference(r) = ty {
        if let Type::Path(p) = r.elem.as_ref() {
            if p.path.is_ident("str") {
                // replace &str with String
                return syn::parse_quote!(String);
            }
        }
    }
    ty.clone()
}