use argument::DuchessDeclaration;
use class_info::{SpannedClassInfo};
use parse::Parser;
use proc_macro::TokenStream;

mod argument;
mod class_info;
mod codegen;
mod parse;
mod reflect;
mod span_error;

/// The main duchess macro, used like so
///
/// ```rust
/// java_package! {
///     package some.pkg.name;
///     class SomeClassName { * }
/// }
/// ```
#[proc_macro]
pub fn java_package(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let decl = match Parser::from(input).parse::<DuchessDeclaration>() {
        Ok(decl) => decl,
        Err(err) => return err.into_tokens().into(),
    };

    match decl.into_tokens() {
        Ok(t) => return t.into(),
        Err(e) => return e.into_tokens().into(),
    }
}

#[proc_macro]
pub fn duchess_javap(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let class_info = match Parser::from(input).parse::<SpannedClassInfo>() {
        Ok(decl) => decl,
        Err(err) => return err.into_tokens().into(),
    };

    class_info.into_tokens().into()
}
