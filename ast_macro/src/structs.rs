use std::cell::RefCell;

use quote::{format_ident, quote};
use syn::{FieldsNamed, Ident, Variant};

use crate::utils::{extract_lifetime::extract_lifetime, innermost_ty::innermost_type};

pub fn struct_defs(
    variant: &Variant,
    fields: &FieldsNamed,
    name: Ident,
) -> proc_macro2::TokenStream {
    let name = format_ident!("{}{}", name, &variant.ident);
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

    let field_names = fields
        .named
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();

    let field_types = fields
        .named
        .iter()
        .map(|f| f.ty.clone())
        .collect::<Vec<_>>();

    let lt = internal_lifetime.borrow().clone();

    quote! {
        #[derive(Debug, PartialEq, Clone, Eq, Hash)]
        pub struct #name #lt {
            #(#struct_fields),*
        }
        impl #lt #name #lt {
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }
    }
}
