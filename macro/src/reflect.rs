use std::{collections::BTreeMap, env, process::Command};

use proc_macro2::TokenStream;

use crate::{
    argument::{DuchessDeclaration, Ident, JavaClass, JavaPackage, JavaPath},
    class_info::{self, Id, SpannedClassInfo, SpannedPackageInfo},
    span_error::SpanError,
};

impl DuchessDeclaration {
    pub fn into_tokens(self) -> Result<TokenStream, SpanError> {
        let spanned_packages = self.to_spanned_packages()?;
        spanned_packages
            .into_values()
            .map(|p| p.into_tokens(0))
            .collect()
    }

    fn to_spanned_packages(&self) -> Result<BTreeMap<Id, SpannedPackageInfo>, SpanError> {
        let mut result = BTreeMap::new();
        for package in &self.packages {
            package.to_spanned_packages(&package.package_name.ids, &mut result)?;
        }
        Ok(result)
    }
}

impl JavaPackage {
    fn to_spanned_packages(
        &self,
        name: &[Ident],
        map: &mut BTreeMap<Id, SpannedPackageInfo>,
    ) -> Result<(), SpanError> {
        let (first, rest) = name.split_first().unwrap();

        let package_info = || SpannedPackageInfo {
            name: first.to_id(),
            span: first.span,
            subpackages: Default::default(),
            classes: Default::default(),
        };

        let first_id = first.to_id();

        // As written, this allows the same package more than once. I don't see any reason to forbid it,
        // but maybe we want to?
        let parent = map.entry(first_id).or_insert_with(package_info);

        if rest.is_empty() {
            for c in &self.classes {
                let j = c.parse_javap(&self.package_name)?;
                parent.classes.push(j);
            }
            Ok(())
        } else {
            self.to_spanned_packages(rest, &mut parent.subpackages)
        }
    }
}

impl JavaClass {
    fn parse_javap(&self, package_name: &JavaPath) -> Result<SpannedClassInfo, SpanError> {
        let mut command = Command::new("javap");

        let classpath = match env::var("CLASSPATH") {
            Ok(val) => val,
            Err(e) => panic!("duchess cannot read the CLASSPATH environment variable: {e}"),
        };

        command
            .arg("-cp")
            .arg(classpath)
            .arg("-public")
            .arg("-s")
            .arg(format!("{}.{}", package_name, self.class_name));

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

        class_info::SpannedClassInfo::parse(&s, self.class_span, self.members.clone())
    }
}
