use std::cell::RefCell;

use quote::{format_ident, quote};
use syn::{FieldsNamed, Variant};

use crate::utils::{extract_lifetime::extract_lifetime, innermost_ty::innermost_type};

pub fn struct_defs(variant: &Variant, fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let name = &variant.ident;
    let formated_name = format_ident!("Expr{}", name);
    let internal_lifetime = RefCell::new(quote! {});

    let struct_fields = fields
        .named
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            let inner_ty = innermost_type(ty);
            let lt = match extract_lifetime(inner_ty) {
                Some(lt) => quote! { <#lt> },
                None => quote! {},
            };
            *internal_lifetime.borrow_mut() = lt.clone();
            quote! { pub #name: #ty }
        })
        .collect::<Vec<_>>();

    let lt = internal_lifetime.borrow().clone();

    quote! {
        #[derive(Debug, PartialEq, Clone)]
        pub struct #formated_name #lt {
            #(#struct_fields),*
        }
    }
}
