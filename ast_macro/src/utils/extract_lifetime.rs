use syn::Lifetime;

pub fn extract_lifetime(ty: &syn::Type) -> Option<&Lifetime> {
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
