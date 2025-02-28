use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FieldsNamed, Variant};

use crate::utils::{extract_lifetime::extract_lifetime, innermost_ty::innermost_type};

pub fn enum_variants(variant: &Variant, fields: &FieldsNamed) -> (TokenStream, TokenStream) {
    let variant_name = format_ident!("{}", &variant.ident);
    let field_name = format_ident!("Expr{}", &variant.ident);

    let lt = fields.named.iter().find_map(|field| {
        let ty = innermost_type(&field.ty);
        extract_lifetime(ty)
    });
    let lt = match lt {
        Some(lt) => {
            quote! {
                <#lt>
            }
        }
        None => {
            quote! {}
        }
    };

    (
        lt.clone(),
        quote! {
            #variant_name(#field_name #lt)
        },
    )
}
