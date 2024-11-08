use anyhow::Context;
use duchess_reflect::{argument::DuchessDeclaration, parse::Parser, reflect::JavapReflector};
use proc_macro2::{Span, TokenStream};

use crate::{files::File, java_package_macro, re};

pub fn process_file(rs_file: &File, reflector: &mut JavapReflector) -> anyhow::Result<bool> {
    let mut watch_file = false;
    for capture in re::java_package().captures_iter(&rs_file.contents) {
        eprintln!("Debug: found java macro in {:?}", rs_file.path);
        let std::ops::Range { start, end: _ } = capture.get(0).unwrap().range();
        java_package_macro::process_macro(reflector, &rs_file, start)
            .with_context(|| format!("failed to process macro {}", rs_file.slug(start)))?;
        watch_file = true;
    }
    Ok(watch_file)
}

fn process_macro(reflector: &mut JavapReflector, file: &File, offset: usize) -> anyhow::Result<()> {
    let the_impl: JavaPackageMacro = match syn::parse_str(file.rust_slice_from(offset))
        .with_context(|| {
            format!(
                "{} failed to parse java_package macro as Rust code",
                file.slug(offset),
            )
        })
        .with_context(|| format!("full contents:\n>>>>{}<<<", file.rust_slice_from(offset)))
    {
        Ok(package) => package,
        Err(e) => {
            // we'll let rustc deal with this later
            eprintln!(
                "Warning: failed to parse java_package macro as Rust code, ignoring it. Error: {}",
                e
            );
            return Ok(());
        }
    };

    let contents = match the_impl.parse_contents() {
        Ok(decl) => decl,
        Err(e) => {
            // we'll let rustc deal with this later
            eprintln!(
                "Warning: failed to parse java_package macro as Duchess code, ignoring it. Error: {}",
                e
            );
            return Ok(());
        }
    };
    cache_all_classes(contents, reflector).with_context(|| "failed to execute javap")?;
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
    fn parse_contents(self) -> anyhow::Result<DuchessDeclaration> {
        let input = self.invocation.mac.tokens;
        Ok(Parser::from(input).parse::<DuchessDeclaration>()?)
    }
}

fn cache_all_classes(
    decl: DuchessDeclaration,
    reflector: &mut JavapReflector,
) -> anyhow::Result<()> {
    let _root_map = decl.to_root_map(reflector)?;
    for class in _root_map.class_names() {
        // forcibly reflect every class
        use duchess_reflect::reflect::Reflect;
        reflector.reflect(&class, Span::call_site())?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::JavaCompiler;
    use duchess_reflect::config::Configuration;
    use tempfile::tempdir;

    #[test]
    fn process_file() {
        let compiler = &JavaCompiler::new(&Configuration::new(), None).unwrap();
        let rs_file = crate::files::File {
            path: "test-files/java_package_1.rs".into(),
            contents: include_str!("../test-files/java_package_1.rs").to_string(),
        };
        super::process_file(&rs_file, &compiler).unwrap();
    }
}
