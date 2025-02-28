use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FieldsNamed, Lifetime, Variant};

use crate::utils::{extract_lifetime::extract_lifetime, innermost_ty::innermost_type};

pub fn enum_variants<'a>(
    variant: &Variant,
    fields: &'a FieldsNamed,
) -> (Option<&'a Lifetime>, TokenStream) {
    let variant_name = format_ident!("{}", &variant.ident);
    let field_name = format_ident!("Expr{}", &variant.ident);

    let lt = fields.named.iter().find_map(|field| {
        let ty = innermost_type(&field.ty);
        extract_lifetime(ty)
    });

    (
        lt,
        quote! {
            #variant_name(#field_name <#lt>)
        },
    )
}
