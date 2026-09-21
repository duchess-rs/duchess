use std::collections::HashSet;

use crate::{
    class_info::{ClassInfo, ClassRef, Constructor, Flags, Method, RefType, RootMap, Type},
    java::is_prelude_class,
    reflect::{JavapClassInfo, PrecomputedReflector},
};

impl RootMap {
    pub fn check(&self, reflector: &PrecomputedReflector) -> syn::Result<()> {
        let mut errors = vec![];

        for class_name in &self.class_names() {
            let ci = self.find_class(class_name).unwrap();
            ci.check(self, reflector, &mut |e| errors.push(e))?;
        }

        match errors.into_iter().reduce(|mut error, next| {
            error.combine(next);
            error
        }) {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

impl ClassInfo {
    fn check(
        &self,
        root_map: &RootMap,
        reflector: &PrecomputedReflector,
        push_error: &mut dyn FnMut(syn::Error),
    ) -> syn::Result<()> {
        let info = reflector.reflect(&self.name, self.span)?;

        let mut push_error_message = |m: String| {
            push_error(syn::Error::new(
                self.span,
                format!("error in class `{}`: {m}", self.name),
            ));
        };

        if self.kind != info.kind {
            push_error_message(format!(
                "class `{}` should be type `{}`",
                self.name,
                format!("{:?}", info.kind).to_lowercase()
            ));
        }

        // We always allow people to elide generics, in which case
        // they are mirroring the "erased" version of the class.
        //
        // We need this (at minimum) to deal with `java.lang.Class`, since we
        // don't want to mirror its parameter.
        if !self.generics.is_empty() {
            // But if there *are* generics, they must match exactly.
            if self.generics != info.generics {
                push_error_message(format!(
                    "class `{}` should have generic parameters `<{}>`",
                    self.name,
                    info.generics
                        .iter()
                        .map(|g| g.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                ));
            }
        }

        for cref in &self.extends {
            if !info.extends.iter().any(|c| c == cref) {
                let extends_list: String = info
                    .extends
                    .iter()
                    .map(|c| format!("`{}`", c))
                    .collect::<Vec<String>>()
                    .join(", ");
                push_error_message(format!(
                    "declared interface `{cref}` not found in the reflected superclasses ({})",
                    extends_list
                ));
            }

            cref.check(root_map, &mut |m| {
                push_error_message(format!("{m}, but is extended by `{}`", self.name))
            });
        }

        // Check whether any extends declarations are duplicates
        error_on_duplicates(self.extends.as_slice(), "extends", &mut push_error_message);

        for cref in &self.implements {
            if !info.implements.iter().any(|c| c == cref) {
                let implements_list: String = info
                    .implements
                    .iter()
                    .map(|c| format!("`{}`", c))
                    .collect::<Vec<String>>()
                    .join(", ");
                push_error_message(format!(
                    "declared interface `{cref}` not found in the reflected interfaces (`{}`)",
                    implements_list
                ));
            }

            cref.check(root_map, &mut |m| {
                push_error_message(format!("{m}, but is implemented by `{}`", self.name));
            });
        }

        // Check whether any implements declarations are duplicates
        error_on_duplicates(
            self.implements.as_slice(),
            "implements",
            &mut push_error_message,
        );

        let refl = JavapClassInfo::from(self.clone());

        for c in &self.constructors {
            let c_method_sig = c.to_method_sig(&refl);

            c.check(root_map, &mut |m| {
                push_error_message(format!(
                    "{m}, which appears in constructor {}",
                    c_method_sig,
                ));
            });

            if !info
                .constructors
                .iter()
                .any(|info_c| info_c.to_method_sig(&info) == c_method_sig)
            {
                push_error_message(format!(
                    "constructor {} does not match any constructors in the reflected class",
                    c_method_sig,
                ));
            }
        }

        for m in &self.methods {
            let m_method_sig = m.to_method_sig();

            let mut push_method_error_message = |msg: String| {
                push_error_message(format!(
                    "{msg}, which appears in method `{}`",
                    m.to_method_sig()
                ));
            };

            m.check(root_map, &mut push_method_error_message);

            if let Some(reflected_m) = info
                .methods
                .iter()
                .find(|info_c| info_c.to_method_sig() == m_method_sig)
            {
                self.compare_flags(m.flags, reflected_m.flags, &mut push_method_error_message);
            } else {
                let same_names: Vec<_> = info
                    .methods
                    .iter()
                    .filter(|info_c| info_c.name == m_method_sig.name)
                    .map(|info_c| info_c.to_method_sig())
                    .map(|info_c| info_c.to_string())
                    .collect();
                if same_names.is_empty() {
                    push_error_message(format!(
                        "no method named `{}` in the reflected class",
                        m_method_sig,
                    ));
                } else {
                    push_error_message(format!(
                        "method `{}` does not match any of the methods in the reflected class: {}",
                        m_method_sig,
                        same_names.join(", "),
                    ));
                }
            }
        }

        Ok(())
    }

    fn compare_flags(
        &self,
        flags: Flags,
        reflected_flags: Flags,
        push_error: &mut dyn FnMut(String),
    ) {
        if self.should_mirror_in_rust(flags.privacy)
            != self.should_mirror_in_rust(reflected_flags.privacy)
        {
            push_error(format!(
                "member declared as {} but it is {} in Java",
                flags.privacy, reflected_flags.privacy,
            ));
        }

        // The is_native flag is only used for compile-time validation and does not
        // affect codegen — duchess invokes all methods via JNI CallXxxMethodA regardless
        // of native status.
        //
        // We allow native-in-Rust + non-native-in-Java because JDK 25 demoted
        // Class.isInterface(), isArray(), and isPrimitive() from native to ordinary
        // methods, and we need to support both JDK ≤21 and JDK 25+.
        //
        // The reverse (non-native-in-Rust + native-in-Java) is still an error because
        // it means the user may need to provide a Rust implementation via
        // #[duchess::java_function] and hasn't marked the method as native.

        if !flags.is_native && reflected_flags.is_native {
            push_error(format!(
                "member not declared as native but it is native in Java",
            ));
        }
    }
}

impl ClassRef {
    fn check(&self, root_map: &RootMap, push_error: &mut dyn FnMut(String)) {
        let (package_name, class_id) = self.name.split();
        if let Some(package) = root_map.find_package(package_name) {
            // A reference to a class from the duchess prelude is fine even
            // when the invocation declares other classes in the same package:
            // codegen resolves each class independently and sends the prelude
            // reference to an absolute `duchess::java::...` path.
            if package.find_class(&class_id).is_none() && !is_prelude_class(&self.name) {
                push_error(format!(
                    "class `{}` not in list of classes to be translated",
                    self.name,
                ))
            }
        }
    }
}

impl Constructor {
    fn check(&self, root_map: &RootMap, mut push_error: impl FnMut(String)) {
        for ty in &self.argument_tys {
            ty.check(root_map, &mut push_error);
        }
    }
}

impl Method {
    fn check(&self, root_map: &RootMap, mut push_error: impl FnMut(String)) {
        for ty in &self.argument_tys {
            ty.check(root_map, &mut push_error);
        }
    }
}

impl Type {
    fn check(&self, root_map: &RootMap, push_error: &mut impl FnMut(String)) {
        match self {
            Type::Ref(r) => r.check(root_map, push_error),
            Type::Scalar(_) => (),
            Type::Repeat(ty) => ty.check(root_map, push_error),
        }
    }
}

impl RefType {
    fn check(&self, root_map: &RootMap, push_error: &mut impl FnMut(String)) {
        match self {
            RefType::Class(cref) => cref.check(root_map, push_error),
            RefType::Array(array) => array.check(root_map, push_error),
            RefType::TypeParameter(_) => (),
            RefType::Extends(ty) => ty.check(root_map, push_error),
            RefType::Super(ty) => ty.check(root_map, push_error),
            RefType::Wildcard => (),
        }
    }
}

fn error_on_duplicates(
    references: &[ClassRef],
    ref_type: &str,
    mut push_error: impl FnMut(String),
) {
    let mut seen = HashSet::with_capacity(references.len());
    for class_ref in references.iter() {
        if seen.contains(&(class_ref.name)) {
            push_error(format!(
                "duplicate reference in '{}' to '{}'",
                ref_type, class_ref.name
            ));
        } else {
            seen.insert(class_ref.name.clone());
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use proc_macro2::Span;

    use super::*;
    use crate::{
        class_info::{DotId, Id, SpannedPackageInfo},
        upcasts::Upcasts,
    };

    /// A `RootMap` declaring `package java.util` with exactly `declared` in
    /// it, mirroring an invocation that declares some (but not all) classes
    /// of a `java.*` package.
    fn root_declaring_java_util(declared: &[&str]) -> RootMap {
        let java_util = SpannedPackageInfo {
            name: Id::from("util"),
            span: Span::call_site(),
            subpackages: BTreeMap::new(),
            classes: declared.iter().map(|name| DotId::parse(*name)).collect(),
        };
        let java = SpannedPackageInfo {
            name: Id::from("java"),
            span: Span::call_site(),
            subpackages: BTreeMap::from([(Id::from("util"), java_util)]),
            classes: Vec::new(),
        };
        RootMap {
            subpackages: BTreeMap::from([(Id::from("java"), java)]),
            classes: BTreeMap::new(),
            upcasts: Upcasts::default(),
        }
    }

    fn check_errors(root: &RootMap, name: &str) -> Vec<String> {
        let class_ref = ClassRef {
            name: DotId::parse(name),
            generics: Vec::new(),
        };
        let mut errors = Vec::new();
        class_ref.check(root, &mut |message| errors.push(message));
        errors
    }

    #[test]
    fn declared_class_passes() {
        let root = root_declaring_java_util(&["java.util.Set"]);
        assert!(check_errors(&root, "java.util.Set").is_empty());
    }

    #[test]
    fn undeclared_prelude_class_passes_alongside_local_package() {
        // The invocation declares `java.util` (via `Set`) while the reference
        // resolves to the duchess prelude.
        let root = root_declaring_java_util(&["java.util.Set"]);
        assert!(check_errors(&root, "java.util.List").is_empty());
    }

    #[test]
    fn undeclared_non_prelude_class_still_fails() {
        let root = root_declaring_java_util(&["java.util.Set"]);
        let errors = check_errors(&root, "java.util.Nope");
        assert_eq!(
            errors,
            ["class `java.util.Nope` not in list of classes to be translated"],
        );
    }

    #[test]
    fn reference_outside_declared_packages_passes() {
        let root = root_declaring_java_util(&["java.util.Set"]);
        assert!(check_errors(&root, "com.example.Foo").is_empty());
    }
}
