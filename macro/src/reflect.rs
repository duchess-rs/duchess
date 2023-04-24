use std::{collections::BTreeMap, env, process::Command, sync::Arc};

use proc_macro2::Span;

use crate::{
    argument::{DuchessDeclaration, Ident, JavaPackage},
    class_info::{ClassDecl, ClassInfo, DotId, Id, RootMap, SpannedPackageInfo},
    span_error::SpanError,
};

impl DuchessDeclaration {
    pub fn to_root_map(&self, reflector: &mut Reflector) -> Result<RootMap, SpanError> {
        let mut subpackages = BTreeMap::new();
        let mut classes = BTreeMap::new();
        for package in &self.packages {
            package.to_spanned_packages(
                &package.package_name.ids,
                reflector,
                &mut subpackages,
                &mut classes,
            )?;
        }
        Ok(RootMap {
            subpackages,
            classes,
        })
    }
}

impl JavaPackage {
    fn to_spanned_packages(
        &self,
        name: &[Ident],
        reflector: &mut Reflector,
        map: &mut BTreeMap<Id, SpannedPackageInfo>,
        classes: &mut BTreeMap<DotId, Arc<ClassInfo>>,
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
            self.insert_classes_into_root_map(reflector, parent, classes)
        } else {
            self.to_spanned_packages(rest, reflector, &mut parent.subpackages, classes)
        }
    }

    fn insert_classes_into_root_map(
        &self,
        reflector: &mut Reflector,
        package: &mut SpannedPackageInfo,
        classes: &mut BTreeMap<DotId, Arc<ClassInfo>>,
    ) -> Result<(), SpanError> {
        for c in &self.classes {
            let (dot_id, info) = match c {
                ClassDecl::Reflected(c) => {
                    let dot_id = self.make_absolute_dot_id(c.span, &c.name)?;
                    let info = reflector.reflect(&dot_id, c.span)?;
                    (dot_id, info.clone())
                }
                ClassDecl::Specified(c) => {
                    let dot_id = self.make_absolute_dot_id(c.span, &c.name)?;
                    (
                        dot_id.clone(),
                        Arc::new(ClassInfo {
                            name: dot_id,
                            ..c.clone()
                        }),
                    )
                }
            };

            package.classes.push(dot_id.clone());
            classes.insert(dot_id, info);
        }
        Ok(())
    }

    /// The users give classnames that may not include java package information.
    fn make_absolute_dot_id(&self, span: Span, class_dot_id: &DotId) -> Result<DotId, SpanError> {
        let package_ids: Vec<Id> = self.package_name.ids.iter().map(|n| n.to_id()).collect();

        let (package, class) = class_dot_id.split();

        // If the user just wrote (e.g.) `String`, add the `java.lang` ourselves.
        if package.is_empty() {
            return Ok(DotId::new(&package_ids, &class));
        }

        // Otherwise, check that the package the user wrote matches our name.
        if &package_ids[..] != package {
            return Err(SpanError {
                span,
                message: format!(
                    "class `{}` expected to be in package `{}`",
                    class_dot_id, self.package_name
                ),
            });
        }

        Ok(class_dot_id.clone())
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
    pub fn reflect(
        &mut self,
        class_name: &DotId,
        span: Span,
    ) -> Result<&Arc<ClassInfo>, SpanError> {
        // yields an error if we cannot reflect on that class.
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
            .arg(format!("{}", class_name));

        let output_or_err = command.output();

        let output = match output_or_err {
            Ok(o) => o,
            Err(err) => {
                return Err(SpanError {
                    span,
                    message: format!("failed to execute `{command:?}`: {err}"),
                });
            }
        };

        if !output.status.success() {
            return Err(SpanError {
                span,
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
                    span,
                    message: format!("failed to parse output of `{command:?}` as utf-8: {err}"),
                });
            }
        };

        let mut ci = ClassInfo::parse(&s, span)?;

        // reset the span for the cached data to the call site so that when others look it up,
        // they get the same span.
        ci.span = Span::call_site();
        Ok(self
            .classes
            .entry(class_name.clone())
            .or_insert(Arc::new(ci)))
    }
}
