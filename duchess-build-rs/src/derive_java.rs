use duchess_reflect::{
    argument::MethodSelector,
    parse::{Parse, Parser},
};
use proc_macro2::Span;
use syn::{parse::ParseStream, Attribute};

use crate::{log, re};

/// Process a file and reflect any `#[java(...)]` attributes that were found
pub(crate) fn process_file(
    rs_file: &crate::files::File,
    reflector: &mut duchess_reflect::reflect::JavapReflector,
) -> anyhow::Result<bool> {
    let mut watch_file = false;
    for capture in re::java_derive().captures_iter(&rs_file.contents) {
        let std::ops::Range { start, end: _ } = capture.get(0).unwrap().range();
        log!(
            "Found derive(java) in {}:{}",
            rs_file.path.display(),
            rs_file.contents[..start].lines().count()
        );
        let derive_java_attr: DeriveJavaAttr = match syn::parse_str(rs_file.rust_slice_from(start))
        {
            Ok(attr) => attr,
            Err(e) => {
                log!("Error: failed to parse derive(java) attribute: {}", e);
                return Ok(true);
            }
        };
        reflector.reflect_and_cache(
            &derive_java_attr.method_selector.class_name(),
            Span::call_site(),
        )?;
        watch_file = true;
    }
    Ok(watch_file)
}

/// Representation of attributes like `#[java(java.lang.Long)]`
#[derive(Debug)]
struct DeriveJavaAttr {
    method_selector: MethodSelector,
}

impl syn::parse::Parse for DeriveJavaAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        for attr in attributes {
            if !attr.path().is_ident("java") {
                continue;
            }
            let derive_tokens = attr.meta.require_list()?.tokens.clone();
            let mut parser: Parser = derive_tokens.into();
            let method_selector = MethodSelector::parse(&mut parser)?.ok_or(syn::Error::new(
                input.span(),
                "expected a class in the attribute",
            ))?;
            return Ok(DeriveJavaAttr { method_selector });
        }
        Err(syn::Error::new(
            input.span(),
            "expected #[java(...)] attribute",
        ))
    }
}
