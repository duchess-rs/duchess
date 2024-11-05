use anyhow::Context;
use duchess_reflect::{argument::DuchessDeclaration, parse::Parser, reflect::Reflector};
use proc_macro2::TokenStream;

use crate::{files::File, java_compiler::JavaCompiler};

pub fn process_macro(compiler: &JavaCompiler, file: &File, offset: usize) -> anyhow::Result<()> {
    let the_impl: JavaPackageMacro = syn::parse_str(file.rust_slice_from(offset))
        .with_context(|| format!("{} failed to parse java_package macro", file.slug(offset),))?;

    the_impl.parse_contents(compiler)?;
    Ok(())
}

struct JavaPackageMacro {
    invocation: syn::ExprMacro,
}

impl syn::parse::Parse for JavaPackageMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // we are parsing an input that starts with an impl and then has add'l stuff
        let invocation: syn::ExprMacro = input.parse()?;

        // syn reports an error if there is anything unconsumed, so consume all remaining tokens
        // after we parse the impl
        let _more_tokens: TokenStream = input.parse()?;

        Ok(Self { invocation })
    }
}

impl JavaPackageMacro {
    fn parse_contents(self, compiler: &JavaCompiler) -> anyhow::Result<()> {
        let input = self.invocation.mac.tokens;
        let decl = Parser::from(input).parse::<DuchessDeclaration>()?;
        let mut reflector = Reflector::new(compiler.configuration());
        let root_map = decl.to_root_map(&mut reflector)?;
        root_map.
        Ok(())
    }
}
