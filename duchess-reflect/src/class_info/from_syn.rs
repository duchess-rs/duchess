use syn::{spanned::Spanned, PathArguments};

use super::{ClassRef, DotId, Id, RefType};

impl ClassRef {
    /// Convert a Rust path (parsed with syn) into a Java path.
    ///
    /// Not the ideal way to express Java types etc but used in
    /// implementing Java interfaces.
    pub fn from(generics: &syn::Generics, path: &syn::Path) -> syn::Result<ClassRef> {
        let syn::Path {
            leading_colon,
            segments,
        } = path;

        if let Some(c) = leading_colon {
            return Err(syn::Error::new(c.span(), "leading colon not accepted"));
        }

        let mut names = vec![];
        let mut types = vec![];
        for segment in segments {
            names.push(segment.ident.clone());

            match &segment.arguments {
                PathArguments::None => {}
                PathArguments::Parenthesized(args) => {
                    return Err(syn::Error::new(
                        args.paren_token.span.open(),
                        "only `<>` type arguments accepted",
                    ));
                }
                PathArguments::AngleBracketed(args) => {
                    if names.len() != segments.len() {
                        return Err(syn::Error::new(
                            args.gt_token.span,
                            "`<>` only permitted in the final segment",
                        ));
                    }

                    for arg in &args.args {
                        let ty = super::RefType::from(generics, arg)?;
                        types.push(ty);
                    }
                }
            }
        }

        Ok(ClassRef {
            name: DotId::from_iter(names.iter().map(|ident| Id::from(ident))),
            generics: types,
        })
    }
}

impl RefType {
    /// Convert a Rust type into a Java type as best we can.
    pub fn from(generics: &syn::Generics, arg: &syn::GenericArgument) -> syn::Result<RefType> {
        if let syn::GenericArgument::Type(ty) = arg {
            if let syn::Type::Path(syn::TypePath { qself, path }) = ty {
                if let Some(q) = qself {
                    return Err(syn::Error::new(q.lt_token.span(), "no qualified paths"));
                }

                for generic in generics.type_params() {
                    if path.is_ident(&generic.ident) {
                        return Ok(RefType::TypeParameter(Id::from(&generic.ident)));
                    }
                }

                Ok(RefType::Class(ClassRef::from(generics, path)?))
            } else {
                Err(syn::Error::new(arg.span(), "only paths accepted"))
            }
        } else {
            Err(syn::Error::new(arg.span(), "only types accepted"))
        }
    }
}
