use quote::ToTokens;
use syn::{GenericArgument, PathArguments, Type};

pub fn innermost_type(ty: &Type) -> &Type {
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
