use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

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
            fn #visitor_method_name(&mut self, node: &#struct_name) -> T;
        });

        // TODO: Check if "ref node" can/should be used here to not consume the node
        accept_match_arms.push(quote! {
            #name::#variant_name(node) => visitor.#visitor_method_name(node)
        });

        structs.push(quote! {
            #vis struct #struct_name {
                #(#struct_fields),*
            }

            impl #struct_name {
                #vis fn new(#(#field_names: #field_types),*) -> Self {
                    Self {
                        #(#field_names),*
                    }
                }
            }
        });

        enum_variants.push(quote! {
            #variant_name(#struct_name)
        });
    }

    let visitor_name = format_ident!("Visitor");
    let visitor_trait = quote! {
        pub trait #visitor_name<T> {
            #(#visitor_methods)*
        }
    };

    let expr_enum = quote! {
        #vis enum #name {
            #(#enum_variants),*
        }

        impl #name {
            #vis fn accept<T, R: #visitor_name<T>>(&self, visitor: &mut R) -> T {
                match self {
                    #(#accept_match_arms),*
                }
            }
        }
    };

    let expanded = quote! {
        #visitor_trait

        #(
            #[derive(Debug)]
            #structs
        )*

        #[derive(Debug)]
        #expr_enum
    };

    TokenStream::from(expanded)
}
