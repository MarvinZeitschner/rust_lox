use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Ast)]
pub fn ast_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;

    let Data::Enum(data_enum) = input.data else {
        panic!("#[derive(Ast)] can only be used on enums");
    };

    let mut structs = vec![];
    let mut enum_variants = vec![];

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let Fields::Named(fields) = &variant.fields else {
            panic!("Enum variants must have named fields");
        };

        let struct_name =
            syn::Ident::new(&format!("{}{}", name, variant_name), variant_name.span());

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

    let enum_name = syn::Ident::new(&format!("{}Ast", name), name.span());

    let expanded = quote! {
        #(#structs)*

        #vis enum #enum_name{
            #(#enum_variants),*
        }
    };

    TokenStream::from(expanded)
}
