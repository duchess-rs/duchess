use std::{
    collections::BTreeMap,
    path::{absolute, Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context};
use proc_macro2::Span;
use serde::{Deserialize, Serialize};

use crate::{
    argument::{DuchessDeclaration, Ident, JavaPackage, MethodSelector},
    class_info::{
        ClassDeclKind, ClassInfo, ClassInfoAccessors, ClassKind, ClassRef, Constructor, DotId,
        Field, Flags, Generic, Id, Method, RootMap, SpannedPackageInfo, Type,
    },
    upcasts::Upcasts,
};

#[cfg(feature = "javap-reflection")]
mod javap;
#[cfg(feature = "javap-reflection")]
pub use javap::JavapReflector;

impl DuchessDeclaration {
    pub fn to_root_map(&self, reflector: &mut dyn Reflect) -> syn::Result<RootMap> {
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

        let upcasts: Upcasts = Upcasts::from_iter(classes.values().map(|v| &**v));

        Ok(RootMap {
            subpackages,
            classes,
            upcasts,
        })
    }
}

impl JavaPackage {
    fn to_spanned_packages(
        &self,
        name: &[Ident],
        reflector: &mut dyn Reflect,
        map: &mut BTreeMap<Id, SpannedPackageInfo>,
        classes: &mut BTreeMap<DotId, Arc<ClassInfo>>,
    ) -> syn::Result<()> {
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
        reflector: &mut dyn Reflect,
        package: &mut SpannedPackageInfo,
        classes: &mut BTreeMap<DotId, Arc<ClassInfo>>,
    ) -> syn::Result<()> {
        for c in &self.classes {
            let (dot_id, info) = match &c.kind {
                ClassDeclKind::Reflected(c) => {
                    let dot_id = self.make_absolute_dot_id(c.span, &c.name)?;
                    let info = reflector.reflect(&dot_id, c.span)?;

                    // We copy over the span and kind for proper error specification and error checking
                    (
                        dot_id,
                        Arc::new(ClassInfo {
                            kind: c.kind,
                            ..info.to_class_info(c.span)
                        }),
                    )
                }
                ClassDeclKind::Specified(c) => {
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
    fn make_absolute_dot_id(&self, span: Span, class_dot_id: &DotId) -> syn::Result<DotId> {
        let package_ids: Vec<Id> = self.package_name.ids.iter().map(|n| n.to_id()).collect();

        let (package, class) = class_dot_id.split();

        // If the user just wrote (e.g.) `String`, add the `java.lang` ourselves.
        if package.is_empty() {
            return Ok(DotId::new(&package_ids, &class));
        }

        // Otherwise, check that the package the user wrote matches our name.
        if &package_ids[..] != package {
            return Err(syn::Error::new(
                span,
                format!("expected package `{}`", self.package_name),
            ));
        }

        Ok(class_dot_id.clone())
    }
}

/// Reflector parsed from a JSON file
#[derive(Debug)]
pub struct PrecomputedReflector {
    classes: BTreeMap<DotId, Arc<JavapClassInfo>>,
}

fn reflection_cache(out_dir: impl AsRef<Path>) -> PathBuf {
    out_dir.as_ref().join("reflection-cache.json")
}

pub trait Reflect {
    fn reflect(&mut self, dot_id: &DotId, span: Span) -> syn::Result<Arc<JavapClassInfo>>;
}

impl Reflect for PrecomputedReflector {
    fn reflect(&mut self, dot_id: &DotId, span: Span) -> syn::Result<Arc<JavapClassInfo>> {
        PrecomputedReflector::reflect(&self, dot_id, span)
    }
}

fn reflect_method(
    class_info: Arc<ClassInfo>,
    method_selector: &MethodSelector,
) -> syn::Result<ReflectedMethod> {
    match method_selector {
        MethodSelector::ClassName(cn) => match class_info.constructors.len() {
            1 => Ok(ReflectedMethod::Constructor(class_info, 0)),
            0 => Err(syn::Error::new(
                cn.span,
                "no constructors found".to_string(),
            )),
            n => Err(syn::Error::new(
                cn.span,
                format!(
                    "{n} constructors found, use an explicit class declaration to disambiguate"
                ),
            )),
        },
        MethodSelector::MethodName(cn, mn) => {
            let methods: Vec<(MethodIndex, &Method)> = class_info
                .methods
                .iter()
                .enumerate()
                .filter(|(_i, m)| &m.name[..] == &mn.text[..])
                .collect();
            match methods.len() {
                    1 => {
                        let (id, _method) = methods[0];
                        Ok(ReflectedMethod::Method(class_info, id))
                    },
                    0 => Err(syn::Error::new(cn.span,  format!("no methods named `{mn}` found"))),
                    n => Err(syn::Error::new(cn.span, format!("{n} methods named `{mn}` found, use an explicit class declaration to disambiguate") )),
                }
        }
        MethodSelector::ClassInfo(_) => todo!(),
    }
}

impl PrecomputedReflector {
    pub fn new() -> anyhow::Result<Self> {
        let Ok(out_dir) = std::env::var("DUCHESS_OUT_DIR") else {
            bail!("DUCHESS_OUT_DIR not set");
        };
        let out_dir = std::path::Path::new(&out_dir);
        if !out_dir.exists() {
            bail!("DUCHESS_OUT_DIR does not exist: {out_dir:?}");
        }
        Self::new_from_path(reflection_cache(out_dir))
    }

    pub fn new_from_path(serialized_class: impl AsRef<Path>) -> anyhow::Result<Self> {
        let abs_path = absolute(&serialized_class);
        Self::new_from_contents(
            &std::fs::read(serialized_class)
                .with_context(|| format!("loading reflection cache from {:?}", abs_path))?,
        )
    }

    pub fn new_from_contents(contents: &[u8]) -> anyhow::Result<Self> {
        let classes = serde_json::from_slice(contents)
            .context("deserializing serialized reflection cache")?;
        Ok(Self { classes })
    }
    /// Returns the (potentially cached) info about `class_name`;
    pub fn reflect(&self, class_name: &DotId, span: Span) -> syn::Result<Arc<JavapClassInfo>> {
        // yields an error if we cannot reflect on that class.
        self.classes.get(class_name).map(Arc::clone).ok_or_else(|| {
            syn::Error::new(
                span,
                format!("no reflected value for `{class_name}` this is a bug"),
            )
        })
    }

    pub fn reflect_method(&self, method_selector: &MethodSelector) -> syn::Result<ReflectedMethod> {
        let class_info = self
            .reflect(&method_selector.class_name(), method_selector.class_span())?
            .to_class_info(method_selector.class_span());
        Ok(reflect_method(Arc::new(class_info), method_selector)?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavapClassInfo {
    pub flags: Flags,
    pub name: DotId,
    pub kind: ClassKind,
    pub generics: Vec<Generic>,
    pub extends: Vec<ClassRef>,
    pub implements: Vec<ClassRef>,
    pub constructors: Vec<Constructor>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

impl ClassInfoAccessors for JavapClassInfo {
    fn flags(&self) -> &Flags {
        &self.flags
    }

    fn name(&self) -> &DotId {
        &self.name
    }

    fn kind(&self) -> ClassKind {
        self.kind
    }

    fn generics(&self) -> &Vec<Generic> {
        &self.generics
    }

    fn extends(&self) -> &Vec<ClassRef> {
        &self.extends
    }

    fn implements(&self) -> &Vec<ClassRef> {
        &self.implements
    }

    fn constructors(&self) -> &Vec<Constructor> {
        &self.constructors
    }

    fn fields(&self) -> &Vec<Field> {
        &self.fields
    }

    fn methods(&self) -> &Vec<Method> {
        &self.methods
    }
}

impl From<ClassInfo> for JavapClassInfo {
    fn from(ci: ClassInfo) -> Self {
        Self {
            flags: ci.flags,
            name: ci.name,
            kind: ci.kind,
            generics: ci.generics,
            extends: ci.extends,
            implements: ci.implements,
            constructors: ci.constructors,
            fields: ci.fields,
            methods: ci.methods,
        }
    }
}

impl JavapClassInfo {
    pub fn to_class_info(&self, span: Span) -> ClassInfo {
        ClassInfo {
            span: span,
            flags: self.flags,
            name: self.name.clone(),
            kind: self.kind,
            generics: self.generics.clone(),
            extends: self.extends.clone(),
            implements: self.implements.clone(),
            constructors: self.constructors.clone(),
            fields: self.fields.clone(),
            methods: self.methods.clone(),
        }
    }
}

pub type ConstructorIndex = usize;
pub type MethodIndex = usize;

/// Reflection on something callable.
#[derive(Clone, Debug)]
pub enum ReflectedMethod {
    Constructor(Arc<ClassInfo>, ConstructorIndex),
    Method(Arc<ClassInfo>, MethodIndex),
}

impl ReflectedMethod {
    /// The name of this callable thing in Rust
    pub fn name(&self) -> Id {
        match self {
            ReflectedMethod::Constructor(..) => Id::from("new"),
            ReflectedMethod::Method(c, m) => c.methods[*m].name.clone(),
        }
    }

    pub fn class(&self) -> &ClassInfo {
        match self {
            ReflectedMethod::Constructor(c, _) => c,
            ReflectedMethod::Method(c, _) => c,
        }
    }

    /// Is this something that is called on a *type*?
    pub fn is_static(&self) -> bool {
        match self {
            ReflectedMethod::Constructor(..) => true,
            ReflectedMethod::Method(c, m) => c.methods[*m].flags.is_static,
        }
    }

    pub fn generics(&self) -> &Vec<Generic> {
        match self {
            ReflectedMethod::Constructor(c, t) => &c.constructors[*t].generics,
            ReflectedMethod::Method(c, m) => &c.methods[*m].generics,
        }
    }

    pub fn argument_tys(&self) -> &Vec<Type> {
        match self {
            ReflectedMethod::Constructor(c, t) => &c.constructors[*t].argument_tys,
            ReflectedMethod::Method(c, m) => &c.methods[*m].argument_tys,
        }
    }
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    use crate::class_info::DotId;
    use crate::reflect::Configuration;

    use super::{JavapReflector, PrecomputedReflector, Reflector};

    #[test]
    fn reflector_rountrips() {
        let mut reflector = Reflector::new_javap(&Configuration::default());
        let _class = reflector
            .reflect_and_cache(
                &DotId::parse("java.lang.String"),
                proc_macro2::Span::call_site(),
            )
            .unwrap();

        let serialized = reflector.serialize();
        let parsed = PrecomputedReflector::new_from_contents(serialized.as_bytes());
        assert_eq!(parsed.classes.len(), 1);
    }
}
