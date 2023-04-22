use std::{collections::BTreeMap, env, process::Command, sync::Arc};

use crate::{
    argument::{DuchessDeclaration, Ident, JavaClass, JavaPackage, JavaPath},
    class_info::{ClassInfo, DotId, Id, RootMap, SpannedClassInfo, SpannedPackageInfo},
    span_error::SpanError,
};

impl DuchessDeclaration {
    pub fn to_root_map(&self, reflector: &mut Reflector) -> Result<RootMap, SpanError> {
        let mut subpackages = BTreeMap::new();
        for package in &self.packages {
            package.to_spanned_packages(&package.package_name.ids, reflector, &mut subpackages)?;
        }
        Ok(RootMap { subpackages })
    }
}

impl JavaPackage {
    fn to_spanned_packages(
        &self,
        name: &[Ident],
        reflector: &mut Reflector,
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
                let j = c.parse_javap(&self.package_name, reflector)?;
                parent.classes.push(j);
            }
            Ok(())
        } else {
            self.to_spanned_packages(rest, reflector, &mut parent.subpackages)
        }
    }
}

/// Reflection cache. Given fully qualified java class names,
/// look up info about their interfaces.
#[derive(Default)]
pub struct Reflector {
    classes: BTreeMap<DotId, Arc<ClassInfo>>,
}

impl Reflector {
    /// Returns the (potentially cached) info about `class_name`;
    /// yields an error if we cannot reflect on that class.
    pub fn reflect(&mut self, class_name: &DotId) -> Result<&Arc<ClassInfo>, String> {
        if self.classes.contains_key(class_name) {
            return Ok(&self.classes[class_name]);
        }

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
            .arg(format!("{}", class_name));

        let output_or_err = command.output();

        let output = match output_or_err {
            Ok(o) => o,
            Err(err) => {
                return Err(format!("failed to execute `{command:?}`: {err}"));
            }
        };

        if !output.status.success() {
            return Err(format!(
                "unsuccessful execution of `{command:?}`: {}",
                String::from_utf8(output.stderr).unwrap_or(String::from("error"))
            ));
        }

        let s = match String::from_utf8(output.stdout) {
            Ok(o) => o,
            Err(err) => {
                return Err(format!(
                    "failed to parse output of `{command:?}` as utf-8: {err}"
                ));
            }
        };

        let ci = ClassInfo::parse(&s)?;
        Ok(self
            .classes
            .entry(class_name.clone())
            .or_insert(Arc::new(ci)))
    }
}

impl JavaClass {
    fn parse_javap(
        &self,
        package_name: &JavaPath,
        reflector: &mut Reflector,
    ) -> Result<SpannedClassInfo, SpanError> {
        let class_name = DotId::parse(format!("{}.{}", package_name, self.class_name));
        match reflector.reflect(&class_name) {
            Ok(s) => Ok(SpannedClassInfo {
                info: ClassInfo::clone(s),
                span: self.class_span,
                members: self.members.clone(),
            }),

            Err(message) => Err(SpanError {
                span: self.class_span,
                message,
            }),
        }
    }
}
