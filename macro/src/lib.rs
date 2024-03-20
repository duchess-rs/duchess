use argument::{DuchessDeclaration, MethodSelector};
use parse::Parser;
use proc_macro::TokenStream;
use rust_format::Formatter;
use std::path::PathBuf;

mod argument;
mod check;
mod class_info;
mod codegen;
mod derive;
mod java_function;
mod parse;
mod reflect;
mod signature;
mod substitution;
mod upcasts;

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

lazy_static::lazy_static! {
    static ref DEBUG_DIR: PathBuf = {
        let tmp_dir = tempfile::TempDir::new().expect("failed to create temp directory");
        tmp_dir.into_path()
    };
}

fn debug_tokens(name: impl std::fmt::Display, token_stream: &proc_macro2::TokenStream) {
    let Ok(debug_filter) = std::env::var("DUCHESS_DEBUG") else {
        return;
    };
    let name = name.to_string();
    let debug_enabled = match debug_filter {
        f if f.eq_ignore_ascii_case("true") || f.eq_ignore_ascii_case("1") => true,
        filter => name.starts_with(&filter)
    };
    if debug_enabled {
        let path = DEBUG_DIR.join(name.replace('.', "_")).with_extension("rs");
        match rust_format::RustFmt::default().format_tokens(token_stream.clone()) {
            Ok(formatted_tokens) => {
                std::fs::write(&path, formatted_tokens).expect("failed to write to debug file");
            }
            Err(_) => {
                std::fs::write(&path, format!("{token_stream:?}")).expect("failed to write to debug file");
            }
        }
        eprintln!("file:///{}", path.display())
    }
}
