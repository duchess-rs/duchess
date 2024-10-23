use std::{fmt::Display, io::Write};

use duchess_reflect::reflect::JavapClassInfo;

use crate::code_writer::CodeWriter;

pub struct ShimWriter<'w> {
    cw: CodeWriter<'w>,
    shim_name: &'w str,
    java_interface_info: &'w JavapClassInfo,
}

impl<'w> ShimWriter<'w> {
    pub fn new(
        writer: &'w mut impl Write,
        shim_name: &'w str,
        java_interface_info: &'w JavapClassInfo,
    ) -> Self {
        ShimWriter {
            cw: CodeWriter::new(writer),
            shim_name,
            java_interface_info,
        }
    }

    pub fn emit_shim_class(mut self) -> anyhow::Result<()> {
        write!(self.cw, "package duchess;")?;

        write!(
            self.cw,
            "public class {} implements {} {{",
            self.shim_name, self.java_interface_info.name
        )?;

        write!(self.cw, "long nativePointer;")?;
        write!(
            self.cw,
            "static java.lang.ref.Cleaner cleaner = java.lang.ref.Cleaner.create();"
        )?;

        write!(self.cw, "public {}(long nativePointer) {{", self.shim_name)?;
        write!(self.cw, "this.nativePointer = nativePointer;")?;
        write!(
            self.cw,
            "cleaner.register(this, () -> {{ native$drop(nativePointer); }});"
        )?;
        write!(self.cw, "}}")?;

        write!(
            self.cw,
            "native static void native$drop(long nativePointer);"
        )?;

        for method in &self.java_interface_info.methods {
            if !method.generics.is_empty() {
                anyhow::bail!(
                    "generic parameters on method `{}` are not supported",
                    method.name
                )
            }

            let native_method_name = format!("native${}", method.name);
            let return_ty: &dyn Display = if let Some(return_ty) = &method.return_ty {
                return_ty
            } else {
                &"void"
            };

            // Emit a native method
            write!(self.cw, "native static {return_ty} {native_method_name}(")?;
            for (argument_ty, index) in method.argument_tys.iter().zip(0..) {
                write!(self.cw, "{argument_ty} arg{index},")?;
            }
            write!(self.cw, "long nativePointer")?;
            write!(self.cw, ");")?;

            // Emit the interface method
            let method_name = &method.name;
            write!(self.cw, "public {return_ty} {method_name}(")?;
            for (argument_ty, index) in method.argument_tys.iter().zip(0..) {
                let comma = if index == method.argument_tys.len() - 1 {
                    ""
                } else {
                    ", "
                };
                write!(self.cw, "{argument_ty} arg{index}{comma}")?;
            }
            write!(self.cw, ") {{")?;
            write!(self.cw, "return {native_method_name}(",)?;
            for index in 0..method.argument_tys.len() {
                write!(self.cw, "arg{index},")?;
            }
            write!(self.cw, "this.nativePointer")?;
            write!(self.cw, ");")?;
            write!(self.cw, "}}")?;
        }

        write!(self.cw, "}}",)?;

        Ok(())
    }
}
