use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

fn type_needs_lifetime(ty: &Type) -> bool {
    match ty {
        // Check for direct references
        Type::Reference(_) => true,
        // Check for types that might contain our Expr type with lifetime
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap();
            // Check if it's Box<Expr> or similar
            if last_segment.ident == "Box" || last_segment.ident == "Expr" {
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn struct_needs_lifetime(fields: &Fields) -> bool {
    fields.iter().any(|field| type_needs_lifetime(&field.ty))
}

#[proc_macro_derive(Ast)]
pub fn ast_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if input.ident == format_ident!("Expr") {
        panic!("Enum can't be named Expr, since it generates this enum");
    }

    let name = format_ident!("Expr");
    let vis = &input.vis;
    let Data::Enum(data_enum) = input.data else {
        panic!("#[derive(Ast)] can only be used on enums");
    };

    let mut structs = vec![];
    let mut enum_variants = vec![];
    let mut visitor_methods = vec![];
    let mut accept_match_arms = vec![];

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let Fields::Named(fields) = &variant.fields else {
            panic!("Enum variants must have named fields");
        };

        let struct_name = format_ident!("{}{}", name, variant_name);
        let needs_lifetime = struct_needs_lifetime(&variant.fields);

        let lifetime_param = if needs_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        let struct_fields = fields.named.iter().map(|f| {
            let field_name = &f.ident;
            let field_type = &f.ty;
            quote! { #vis #field_name: #field_type }
        });

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

        let visitor_method_name =
            format_ident!("visit_{}", variant_name.to_string().to_lowercase());

        visitor_methods.push(quote! {
            fn #visitor_method_name(&mut self, node: &#struct_name #lifetime_param) -> T;
        });

        accept_match_arms.push(quote! {
            Self::#variant_name(node) => visitor.#visitor_method_name(node)
        });

        // Generate struct with lifetime only if needed
        structs.push(quote! {
            #[derive(Debug)]
            #vis struct #struct_name #lifetime_param {
                #(#struct_fields),*
            }

            impl #lifetime_param #struct_name #lifetime_param {
                #vis fn new(#(#field_names: #field_types),*) -> Self {
                    Self {
                        #(#field_names),*
                    }
                }
            }
        });

        // Add lifetime to enum variant only if needed
        enum_variants.push(if needs_lifetime {
            quote! { #variant_name(#struct_name<'a>) }
        } else {
            quote! { #variant_name(#struct_name) }
        });
    }

    let visitor_name = format_ident!("Visitor");

    let visitor_trait = quote! {
        pub trait #visitor_name<'a, T> {
            #(#visitor_methods)*
        }
    };

    let expr_enum = quote! {
        #[derive(Debug)]
        #vis enum #name<'a> {
            #(#enum_variants),*
        }

        impl<'a> #name<'a> {
            #vis fn accept<T, V: #visitor_name<'a, T>>(&self, visitor: &mut V) -> T {
                match self {
                    #(#accept_match_arms),*
                }
            }
        }
    };

    let expanded = quote! {
        #visitor_trait

        #(#structs)*

        #expr_enum
    };

    TokenStream::from(expanded)
}
