use std::cell::RefCell;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lifetime, Lit};

mod enums;
mod structs;
mod utils;

/// Turns an enum with named-field variants into one struct per variant plus an enum wrapping
/// those structs. With `#[name = "Expr"]`, the variant `Binary { .. }` becomes the struct
/// `ExprBinary` and the enum variant `Expr::Binary(ExprBinary)`.
#[proc_macro_derive(Ast, attributes(name))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut name = input.ident;

    for attr in input.attrs {
        if attr.path().is_ident("name") {
            let tokens = attr.meta.require_name_value().unwrap();
            let attr_name = &tokens.value;
            let attr_name = match attr_name {
                Expr::Lit(lit) => match &lit.lit {
                    Lit::Str(s) => s.value(),
                    _ => panic!("Expected a string literal"),
                },
                _ => panic!("Expected a string literal"),
            };
            name = format_ident!("{}", attr_name);
        }
    }

    let Data::Enum(data) = input.data else {
        panic!("#[derive(AST)] can only be used on enums");
    };

    let mut structs: Vec<proc_macro2::TokenStream> = vec![];

    let mut enum_variants: Vec<proc_macro2::TokenStream> = vec![];
    let enum_lifetime: RefCell<Option<&Lifetime>> = RefCell::new(None);

    for variant in &data.variants {
        let Fields::Named(fields) = &variant.fields else {
            panic!("Enum variants must have named fields");
        };
        structs.push(structs::struct_defs(variant, fields, name.clone()));

        let (en_lt, en_variants) = enums::enum_variants(variant, fields, name.clone());
        enum_variants.push(en_variants);
        // A variant without a lifetime must not reset the lifetime found on another variant
        if en_lt.is_some() {
            enum_lifetime.replace(en_lt);
        }
    }

    let enum_lifetime = enum_lifetime.clone().into_inner();

    let enum_lifetime_tokenstream = match enum_lifetime {
        Some(lt) => quote! { <#lt> },
        None => quote! {},
    };

    let expanded = quote! {
        #[derive(Debug, PartialEq, Clone)]
        pub enum #name #enum_lifetime_tokenstream {
            #(#enum_variants),*
        }

        #(#structs)*
    };

    expanded.into()
}
