use syn::{Field, PathArguments, Type};

pub trait IsOption {
    fn is_option(&self) -> bool;
}

impl IsOption for Field {
    fn is_option(&self) -> bool {
        if let Type::Path(type_path) = &self.ty {
            if let Some(last_segment) = type_path.path.segments.last() {
                return last_segment.ident == "Option"
                    && matches!(
                        last_segment.arguments,
                        PathArguments::AngleBracketed(ref args) if !args.args.is_empty()
                    );
            }
        }
        false
    }
}
