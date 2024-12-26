use duchess_reflect::{
    class_info::ClassRef,
    reflect::{JavapReflector, Reflect},
};
use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::{files::File, java_compiler::JavaCompiler, log, shim_writer::ShimWriter};

pub fn process_impl(compiler: &JavaCompiler, file: &File, offset: usize) -> anyhow::Result<()> {
    let the_impl: JavaInterfaceImpl = syn::parse_str(file.rust_slice_from(offset))?;
    the_impl.generate_shim(compiler)?;
    Ok(())
}

struct JavaInterfaceImpl {
    item: syn::ItemImpl,
}

impl syn::parse::Parse for JavaInterfaceImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // we are parsing an input that starts with an impl and then has add'l stuff
        let item: syn::ItemImpl = input.parse()?;

        // syn reports an error if there is anything unconsumed, so consume all remaining tokens
        // after we parse the impl
        let _more_tokens: TokenStream = input.parse()?;

        Ok(Self { item })
    }
}

impl JavaInterfaceImpl {
    fn generate_shim(&self, compiler: &JavaCompiler) -> anyhow::Result<()> {
        let mut reflector = JavapReflector::new(compiler.configuration());
        let (java_interface_ref, java_interface_span) = self.java_interface()?;
        let java_interface_info =
            reflector.reflect(&java_interface_ref.name, java_interface_span)?;

        let shim_name = format!("Shim${}", java_interface_info.name.to_dollar_name());
        let java_file = compiler.java_file("duchess", &shim_name);
        ShimWriter::new(
            &mut java_file.src_writer()?,
            &shim_name,
            &java_interface_info,
        )
        .emit_shim_class()?;

        compiler.compile_to_rs_file(&java_file)?;

        log!("compiled to {}", java_file.rs_path.display());

        Ok(())
    }

    fn java_interface(&self) -> anyhow::Result<(ClassRef, Span)> {
        let Some((_, trait_path, _)) = &self.item.trait_ else {
            return Err(syn::Error::new_spanned(&self.item, "expected an impl of a trait").into());
        };
        let class_ref = ClassRef::from(&self.item.generics, trait_path)?;
        Ok((class_ref, trait_path.span()))
    }
}
