use argument::DuchessDeclaration;
use parse::Parser;
use proc_macro::TokenStream;

mod argument;
mod check;
mod class_info;
mod codegen;
mod parse;
mod reflect;
mod span_error;

/// The main duchess macro, used like so
///
/// ```rust,ignore
/// duchess::java_package! {
///     package some.pkg.name;
///     class SomeClassName { * }
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
        Err(err) => return err.into_tokens().into(),
    }
}
