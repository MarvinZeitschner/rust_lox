use std::cell::RefCell;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lifetime, Lit};

mod enums;
mod structs;
mod utils;
mod visitor;

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

    let mut visitor_methods: Vec<proc_macro2::TokenStream> = vec![];
    let mut accept_methods: Vec<proc_macro2::TokenStream> = vec![];

    for variant in &data.variants {
        let Fields::Named(fields) = &variant.fields else {
            panic!("Enum variants must have named fields");
        };
        structs.push(structs::struct_defs(variant, fields, name.clone()));

        let (en_lt, en_variants) = enums::enum_variants(variant, fields, name.clone());
        enum_variants.push(en_variants);
        enum_lifetime.replace(en_lt);

        visitor_methods.push(visitor::visitor_method(variant, en_lt, name.clone()));
        accept_methods.push(visitor::accept_method(variant));
    }

    let enum_lifetime = enum_lifetime.clone().into_inner();

    let visitor_lifetime_tokenstream = match enum_lifetime {
        Some(lt) => quote! { <#lt, 'b> },
        None => quote! {},
    };

    let enum_lifetime_tokenstream = match enum_lifetime {
        Some(lt) => quote! { <#lt> },
        None => quote! {},
    };

    let visitor_name = format_ident!("{}Visitor", name);

    let visitor_trait = quote! {
        pub trait #visitor_name #visitor_lifetime_tokenstream {
            type Output;

            #(#visitor_methods)*
        }
    };

    let expanded = quote! {
        #visitor_trait

        #[derive(Debug, PartialEq, Clone, Eq, Hash)]
        pub enum #name #enum_lifetime_tokenstream {
            #(#enum_variants),*
        }

        impl #visitor_lifetime_tokenstream #name #enum_lifetime_tokenstream {
            pub fn accept<V: #visitor_name #visitor_lifetime_tokenstream>(&'b self, visitor: &mut V) -> V::Output {
                match self {
                    #(#accept_methods),*
                }
            }
        }

        #(#structs)*
    };

    // panic!("{}", expanded);

    expanded.into()
}
