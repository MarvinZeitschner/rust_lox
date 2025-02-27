use std::cell::RefCell;

use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, Type, Variant};

fn innermost_type(ty: &Type) -> &Type {
    match ty {
        Type::Path(_) => ty,
        Type::Group(group) => innermost_type(&group.elem),
        Type::Paren(paren) => innermost_type(&paren.elem),
        Type::Reference(reference) => innermost_type(&reference.elem),
        Type::Array(array) => innermost_type(&array.elem),
        Type::Tuple(tuple) if !tuple.elems.is_empty() => innermost_type(&tuple.elems[0]),
        Type::Infer(_) => ty,
        _ => ty,
    }
}
fn struct_defs(variant: &Variant, fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let name = &variant.ident;
    let formated_name = format_ident!("Expr{}", name);
    let internal_lifetime = RefCell::new(quote! {});

    let struct_fields = fields
        .named
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            let ty = innermost_type(ty);
            panic!("{}", ty.to_token_stream().to_string());
            let lt = match ty {
                Type::Reference(ty) => match &ty.lifetime {
                    Some(lt) => {
                        internal_lifetime.replace(quote! { <#lt> });
                        quote! { <#lt> }
                    }
                    None => quote! {},
                },
                _ => quote! {},
            };
            quote! { pub #name: #ty #lt }
        })
        .collect::<Vec<_>>();

    let lt = internal_lifetime.borrow().clone();
    // panic!("{}", lt);

    quote! {
        #[derive(Debug, PartialEq, Clone)]
        pub struct #formated_name #lt {
            #(#struct_fields),*
        }
    }
}

#[proc_macro_derive(Ast)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Enum(data) = input.data else {
        panic!("#[derive(AST)] can only be used on enums");
    };

    let mut structs: Vec<proc_macro2::TokenStream> = vec![];

    for variant in &data.variants {
        let Fields::Named(fields) = &variant.fields else {
            panic!("Enum variants must have named fields");
        };
        structs.push(struct_defs(variant, fields));
    }

    let expanded = quote! {
        #(#structs)*
    };

    panic!("{}", expanded);

    expanded.into()
}
