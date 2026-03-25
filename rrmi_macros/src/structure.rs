use quote::quote;
use std::fmt::Debug;
use syn::{
    FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, ReturnType, Token, Type,
    punctuated::Punctuated,
};

use crate::utils::fix_case;

pub struct RemoteObjectInfo {
    pub struct_name: StructNameInfo,
    pub methods: Vec<RemoteMethodInfo>,
}

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

pub struct StructNameInfo(pub Ident);

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

pub struct RemoteMethodInfo {
    pub name: Ident,
    pub params: ParametersInfo,
    pub ret: ReturnType,
}

impl RemoteMethodInfo {
    pub fn get_name_fixed(&self) -> Ident {
        let name = &self.name;
        Ident::new(&fix_case(&name.to_string()), name.span().clone())
    }

    pub fn get_ret(&self) -> Type {
        match &self.ret {
            ReturnType::Default => syn::parse_quote!(()),
            ReturnType::Type(_, ty) => *ty.clone(),
        }
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

pub struct ParametersInfo(pub Vec<ParameterInfo>);

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

pub struct ParameterInfo(pub (Ident, Type));

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
