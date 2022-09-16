use quote::quote;
use syn::{Fields, Generics, Ident, ItemStruct, Result, Type, GenericArgument};

use crate::{attr::StructAttr, utils::to_ts_ident, DerivedTS};

mod r#enum;
mod generics;
mod named;
mod newtype;
mod tuple;
mod unit;

pub(crate) use r#enum::enum_def;

pub(crate) fn struct_def(s: &ItemStruct) -> Result<DerivedTS> {
    let attr = StructAttr::from_attrs(&s.attrs)?;

    type_def(&attr, &s.ident, &s.fields, &s.generics)
}

fn type_def(
    attr: &StructAttr,
    ident: &Ident,
    fields: &Fields,
    generics: &Generics,
) -> Result<DerivedTS> {
    let name = attr.rename.clone().unwrap_or_else(|| to_ts_ident(ident));
    match fields {
        Fields::Named(named) => match named.named.len() {
            0 => unit::unit(attr, &name),
            _ => named::named(attr, &name, named, generics),
        },
        Fields::Unnamed(unnamed) => match unnamed.unnamed.len() {
            0 => unit::unit(attr, &name),
            1 => newtype::newtype(attr, &name, unnamed, generics),
            _ => tuple::tuple(attr, &name, unnamed, generics),
        },
        Fields::Unit => unit::unit(attr, &name),
    }
}

pub fn make_lifetimes_static(ty: &Type) -> Type {
    // Remap all lifetimes to 'static in ty.
    struct Visitor;
    impl syn::visit_mut::VisitMut for Visitor {
        fn visit_type_mut(&mut self, ty: &mut Type) {
            match ty {
                Type::Reference(ref_type) => {
                    ref_type.lifetime = ref_type
                        .lifetime
                        .as_ref()
                        .map(|_| syn::parse2(quote!('static)).unwrap());
                }
                _ => {}
            }
            syn::visit_mut::visit_type_mut(self, ty);
        }

        fn visit_generic_argument_mut(&mut self, ga: &mut GenericArgument) {
            match ga {
                GenericArgument::Lifetime(lt) => {
                    *lt = syn::parse2(quote!('static)).unwrap();
                }
                _ => {}
            }
            syn::visit_mut::visit_generic_argument_mut(self, ga);
        }
    }
    use syn::visit_mut::VisitMut;
    let mut ty = ty.clone();
    Visitor.visit_type_mut(&mut ty);
    ty
}
