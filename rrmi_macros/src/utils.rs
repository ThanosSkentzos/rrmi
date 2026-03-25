use syn::Type;

pub fn fix_case(s: &str) -> String {
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

pub fn normalize_type(ty: &Type) -> Type {
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
