use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Lifetime, Variant};

pub fn visitor_method(variant: &Variant, lifetime: Option<&Lifetime>) -> TokenStream {
    let struct_name = format_ident!("Expr{}", &variant.ident);
    let visitor_name = format_ident!("visit_{}", &variant.ident.to_string().to_lowercase());

    let lt = match lifetime {
        Some(lt) => quote! { <#lt> },
        None => quote! {},
    };

    quote! {
        fn #visitor_name(&mut self, node: &#struct_name #lt) -> T;
    }
}
