use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Lifetime, Variant};

pub fn visitor_method(variant: &Variant, lifetime: Option<&Lifetime>, name: Ident) -> TokenStream {
    let struct_name = format_ident!("{}{}", name, &variant.ident);
    let visitor_name = format_ident!("visit_{}", &variant.ident.to_string().to_lowercase());

    let lt = match lifetime {
        Some(lt) => quote! { <#lt> },
        None => quote! {},
    };

    // TODO: This should be dynamically generated to the actual input, but since the macro is just
    // for this project.....
    let sep_lt = match lifetime {
        Some(_) => quote! { 'b },
        None => quote! {},
    };

    quote! {
        fn #visitor_name(&mut self, node: & #sep_lt #struct_name #lt) -> Self::Output;
    }
}

pub fn accept_method(variant: &Variant) -> TokenStream {
    let variant_name = format_ident!("{}", &variant.ident);
    let visitor_name = format_ident!("visit_{}", &variant.ident.to_string().to_lowercase());

    quote! {
        Self::#variant_name(node) => visitor.#visitor_name(node)
    }
}
