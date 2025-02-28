use std::cell::RefCell;

use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, GenericArgument, Lifetime,
    PathArguments, Type, Variant,
};

fn extract_lifetime(ty: &syn::Type) -> Option<&Lifetime> {
    match ty {
        syn::Type::Path(type_path) if !type_path.path.segments.is_empty() => {
            let last_segment = type_path.path.segments.last().unwrap();

            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Lifetime(lifetime) = arg {
                        return Some(lifetime);
                    }
                }
            }
            None
        }
        syn::Type::Reference(type_ref) => type_ref.lifetime.as_ref(),
        _ => None,
    }
}
fn innermost_type(ty: &Type) -> &Type {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            return innermost_type(inner_ty);
                        }
                    }
                }
            }
            ty
        }
        Type::Reference(reference) => innermost_type(&reference.elem),
        _ => panic!("Unexpected type: {:?}", ty.to_token_stream().to_string()),
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
            let lt = match extract_lifetime(ty) {
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

    // panic!("{}", expanded);

    expanded.into()
}
