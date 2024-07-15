use std::path::PathBuf;

use rust_format::Formatter as _;

pub mod argument;
pub mod check;
pub mod class_info;
pub mod codegen;
pub mod parse;
pub mod reflect;
pub mod signature;
pub mod substitution;
pub mod upcasts;

lazy_static::lazy_static! {
    static ref DEBUG_DIR: PathBuf = {
        let tmp_dir = tempfile::TempDir::new().expect("failed to create temp directory");
        tmp_dir.into_path()
    };
}

pub fn debug_tokens(name: impl std::fmt::Display, token_stream: &proc_macro2::TokenStream) {
    let Ok(debug_filter) = std::env::var("DUCHESS_DEBUG") else {
        return;
    };
    let name = name.to_string();
    let debug_enabled = match debug_filter {
        f if f.eq_ignore_ascii_case("true") || f.eq_ignore_ascii_case("1") => true,
        filter => name.starts_with(&filter),
    };
    if debug_enabled {
        let path = DEBUG_DIR.join(name.replace('.', "_")).with_extension("rs");
        match rust_format::RustFmt::default().format_tokens(token_stream.clone()) {
            Ok(formatted_tokens) => {
                std::fs::write(&path, formatted_tokens).expect("failed to write to debug file");
            }
            Err(_) => {
                std::fs::write(&path, format!("{token_stream:?}"))
                    .expect("failed to write to debug file");
            }
        }
        // in JetBrains terminal, links are only clickable with a `file:///` prefix. But in VsCode
        // iTerm, and most other terminals, they are only clickable if they are absolute paths.
        if running_in_jetbrains() {
            eprintln!("file:///{}", path.display())
        } else {
            eprintln!("{}", path.display())
        }
    }
}

fn running_in_jetbrains() -> bool {
    std::env::var("TERMINAL_EMULATOR")
        .map(|var| var.contains("JetBrains"))
        .unwrap_or_default()
}
