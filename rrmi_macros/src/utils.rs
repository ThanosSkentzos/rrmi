use super::TokenStream2;
use quote::quote;
use syn::{Ident, Type};

pub fn camel_case(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
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

pub fn fix_ref_to_type(ty: &Type) -> Type {
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

pub fn fix_ref_when_called(param: &(Ident, Type)) -> TokenStream2 {
    let (ident, ty) = param;
    if let Type::Reference(r) = ty {
        if let Type::Path(p) = r.elem.as_ref() {
            if p.path.is_ident("str") {
                return quote! {#ident: #ident.to_string()};
            }
        }
    }
    quote! {#ident}
}

#[allow(unused)]
pub fn already_rmi_result(ty: &Type) -> bool {
    // take the type and check if is already ::foo::bar::RMIResult<T>
    if let Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if last.ident == "RMIResult" {
                return true;
            }
        }
    }
    false
}

pub fn is_str_ref(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        if let Type::Path(p) = r.elem.as_ref() {
            return p.path.is_ident("str");
        }
    }
    false
}
