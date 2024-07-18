use argument::{DuchessDeclaration, MethodSelector};
use parse::Parser;
use proc_macro::TokenStream;

use duchess_reflect::*;

mod derive;
mod java_function;

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
        Err(err) => return err.to_compile_error().into(),
    };

    match decl.to_tokens() {
        Ok(t) => return t.into(),
        Err(e) => return e.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn java_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let args = match Parser::from(args).parse::<MethodSelector>() {
        Ok(decl) => decl,
        Err(err) => return err.to_compile_error().into(),
    };

    let item_fn = match syn::parse::<syn::ItemFn>(input) {
        Ok(item_fn) => item_fn,
        Err(err) => return err.into_compile_error().into(),
    };

    match java_function::java_function(args, item_fn) {
        Ok(t) => t.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

synstructure::decl_derive!([ToRust, attributes(java)] => derive::derive_to_rust);

synstructure::decl_derive!([ToJava, attributes(java)] => derive::derive_to_java);
