use std::cell::RefCell;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lifetime};

mod enums;
mod structs;
mod utils;
mod visitor;

#[proc_macro_derive(Ast)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

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
        structs.push(structs::struct_defs(variant, fields));

        let (en_lt, en_variants) = enums::enum_variants(variant, fields);
        enum_variants.push(en_variants);
        enum_lifetime.replace(en_lt);

        visitor_methods.push(visitor::visitor_method(variant));
        accept_methods.push(visitor::accept_method(variant));
    }

    let enum_name = format_ident!("Expr");
    let enum_lifetime = enum_lifetime.clone().into_inner();

    let enum_lifetime_tokenstream = match enum_lifetime {
        Some(lt) => quote! { <#lt> },
        None => quote! {},
    };

    let visitor_trait = quote! {
        pub trait Visitor {
            type Output;

            #(#visitor_methods)*
        }
    };

    let expanded = quote! {
        #visitor_trait

        #[derive(Debug, PartialEq, Clone)]
        pub enum #enum_name #enum_lifetime_tokenstream {
            #(#enum_variants),*
        }

        impl #enum_lifetime_tokenstream #enum_name #enum_lifetime_tokenstream {
            pub fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
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
