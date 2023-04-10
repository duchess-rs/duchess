use std::process::Command;

use proc_macro2::TokenStream;

use crate::{
    argument::{DuchessDeclaration, JavaClass, JavaPackage},
    class_info::{self, SpannedClassInfo},
    span_error::SpanError,
};

impl DuchessDeclaration {
    pub fn into_tokens(self) -> Result<TokenStream, SpanError> {
        self.packages.into_iter().map(|p| p.into_tokens()).collect()
    }
}

impl JavaPackage {
    pub fn into_tokens(self) -> Result<TokenStream, SpanError> {
        self.classes
            .iter()
            .map(|c| Ok(c.parse_javap(&self.package_name.text)?.into_tokens()))
            .collect()
    }
}

impl JavaClass {
    fn parse_javap(&self, package_name: &str) -> Result<SpannedClassInfo, SpanError> {
        let mut command = Command::new("javap");

        command
            .arg("-public")
            .arg("-s")
            .arg(format!("{}.{}", package_name, self.class_name))
            .env("CLASSPATH", "java") // FIXME: HACK
            ;

        let output_or_err = command.output();

        let output = match output_or_err {
            Ok(o) => o,
            Err(err) => {
                return Err(SpanError {
                    span: self.class_span,
                    message: format!("failed to execute `{command:?}`: {err}"),
                })
            }
        };

        if !output.status.success() {
            return Err(SpanError {
                span: self.class_span,
                message: format!(
                    "unsuccessful execution of `{command:?}`: {}",
                    String::from_utf8(output.stderr).unwrap_or(String::from("error"))
                ),
            });
        }

        let s = match String::from_utf8(output.stdout) {
            Ok(o) => o,
            Err(err) => {
                return Err(SpanError {
                    span: self.class_span,
                    message: format!("failed to parse output of `{command:?}` as utf-8: {err}"),
                })
            }
        };

        class_info::SpannedClassInfo::parse(&s, self.class_span)
    }
}
