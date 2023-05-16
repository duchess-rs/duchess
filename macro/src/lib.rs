use argument::DuchessDeclaration;
use parse::Parser;
use proc_macro::TokenStream;
use rust_format::Formatter;

mod argument;
mod check;
mod class_info;
mod codegen;
mod derive;
mod parse;
mod reflect;
mod signature;
mod span_error;
mod upcasts;
mod substitution;

/// The main duchess macro, used like so
///
/// ```rust,ignore
/// duchess::java_package! {
///     package some.pkg.name;
///     class SomeDotId { * }
/// }
/// ```
///
/// see the tutorial in the [duchess book] for more info.
///
/// [duchess book]: https://nikomatsakis.github.io/duchess/
#[proc_macro]
pub fn java_package(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let decl = match Parser::from(input).parse::<DuchessDeclaration>() {
        Ok(decl) => decl,
        Err(err) => return err.into_tokens().into(),
    };

    match decl.to_tokens() {
        Ok(t) => return t.into(),
        Err(e) => return e.into_tokens().into(),
    }
}

synstructure::decl_derive!([ToRust, attributes(java)] => derive::derive_to_rust);

synstructure::decl_derive!([ToJava, attributes(java)] => derive::derive_to_java);

fn debug_tokens(name: impl std::fmt::Display, token_stream: &proc_macro2::TokenStream) {
    let Ok(f) = std::env::var("DUCHESS_DEBUG") else { return };
    if f == "*" || f == "1" || name.to_string().starts_with(&f) {
        match rust_format::RustFmt::default().format_tokens(token_stream.clone()) {
            Ok(v) => {
                eprintln!("{v}");
            }
            Err(_) => {
                eprintln!("{token_stream:?}");
            }
        }
    }
}
