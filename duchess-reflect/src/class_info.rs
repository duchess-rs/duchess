use std::{collections::BTreeMap, sync::Arc};

use inflector::Inflector;
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::quote_spanned;
use serde::{Deserialize, Serialize};

use crate::{
    parse::{Parse, TextAccum},
    reflect::JavapClassInfo,
    upcasts::Upcasts,
};

/// Stores all the data about the classes/packages to be translated
/// as well as whatever we have learned from reflection.
#[derive(Debug)]
pub struct RootMap {
    pub subpackages: BTreeMap<Id, SpannedPackageInfo>,
    pub classes: BTreeMap<DotId, Arc<ClassInfo>>,
    pub upcasts: Upcasts,
}

impl RootMap {
    /// Finds the class with the given name (if present).
    pub fn find_class(&self, cn: &DotId) -> Option<&Arc<ClassInfo>> {
        self.classes.get(cn)
    }

    /// Finds the package with the given name (if present).
    pub fn find_package(&self, ids: &[Id]) -> Option<&SpannedPackageInfo> {
        let (p0, ps) = ids.split_first().unwrap();
        self.subpackages.get(p0)?.find_subpackage(ps)
    }

    pub fn to_packages(&self) -> impl Iterator<Item = &SpannedPackageInfo> {
        self.subpackages.values()
    }

    /// Find the names of all classes contained within.
    pub fn class_names(&self) -> Vec<DotId> {
        self.classes.keys().cloned().collect()
    }
}

#[derive(Debug)]
pub struct SpannedPackageInfo {
    pub name: Id,
    pub span: Span,
    pub subpackages: BTreeMap<Id, SpannedPackageInfo>,
    pub classes: Vec<DotId>,
}

impl SpannedPackageInfo {
    /// Find a (sub)package given its relative name
    pub fn find_subpackage(&self, ids: &[Id]) -> Option<&SpannedPackageInfo> {
        let Some((p0, ps)) = ids.split_first() else {
            return Some(self);
        };

        self.subpackages.get(p0)?.find_subpackage(ps)
    }

    /// Finds a class in this package with the given name (if any)
    pub fn find_class(&self, cn: &Id) -> Option<&DotId> {
        self.classes.iter().find(|c| c.is_class(cn))
    }
}

#[derive(Debug)]
pub struct ClassDecl {
    pub kind: ClassDeclKind,
}

#[derive(Debug)]
pub enum ClassDeclKind {
    /// User wrote `class Foo { * }`
    Reflected(ReflectedClassInfo),

    /// User wrote `class Foo { ... }` with full details.
    Specified(ClassInfo),
}

impl Parse for ClassDecl {
    fn parse(p: &mut crate::parse::Parser) -> syn::Result<Option<Self>> {
        // Look for a keyword that could start a class definition.
        let Some(t0) = p.peek_token() else {
            return Ok(None);
        };
        match t0 {
            TokenTree::Ident(i) => {
                static START_KEYWORDS: &[&str] = &[
                    "class",
                    "public",
                    "final",
                    "abstract",
                    "interface",
                    "enum",
                    "record",
                ];
                let s = i.to_string();
                if !START_KEYWORDS.contains(&s.as_str()) {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        }

        // Accumulate tokens until we see a braced block `{}` that is the class body.
        let t0 = p.eat_token().unwrap();
        let mut accum = TextAccum::new(p, t0);
        while let Some(t1) = accum.accum() {
            match t1 {
                TokenTree::Group(d) if d.delimiter() == Delimiter::Brace => {
                    break;
                }
                _ => {}
            }
        }

        // Parse the text with LALRPOP.
        let (text, span) = accum.into_accumulated_result();
        let kind = javap::parse_class_decl(span, &text)?;
        Ok(Some(ClassDecl { kind }))
    }

    fn description() -> String {
        format!("class definition (copy/paste the output from `javap -public`)")
    }
}

#[derive(Clone, Debug)]
pub struct ReflectedClassInfo {
    pub span: Span,
    pub flags: Flags,
    pub name: DotId,
    pub kind: ClassKind,
}

/// ClassInfo that was parsed from a hand written declaration
#[derive(Clone, Debug)]
pub struct ClassInfo {
    pub span: Span,
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

/// Trait that allows code to be generic over [`ClassInfo`][]
/// or [`JavapClassInfo`][].
pub trait ClassInfoAccessors {
    fn flags(&self) -> &Flags;
    fn name(&self) -> &DotId;
    fn kind(&self) -> ClassKind;
    fn generics(&self) -> &Vec<Generic>;
    fn extends(&self) -> &Vec<ClassRef>;
    fn implements(&self) -> &Vec<ClassRef>;
    fn constructors(&self) -> &Vec<Constructor>;
    fn fields(&self) -> &Vec<Field>;
    fn methods(&self) -> &Vec<Method>;

    fn this_ref(&self) -> ClassRef {
        ClassRef {
            name: self.name().clone(),
            generics: self
                .generics()
                .iter()
                .map(|g| RefType::TypeParameter(g.id.clone()))
                .collect(),
        }
    }
}

impl ClassInfoAccessors for ClassInfo {
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

impl ClassInfo {
    pub fn parse(text: &str, span: Span) -> syn::Result<ClassInfo> {
        javap::parse_class_info(span, &text)
    }

    /// Indicates whether a member with the given privacy level should be reflected in Rust.
    /// We always mirror things declared as public.
    /// In classes, the default privacy indicates "package level" visibility and we do not mirror.
    /// In interfaces, the default privacy indicates "public" visibility and we DO mirror.
    pub fn should_mirror_in_rust(&self, privacy: Privacy) -> bool {
        match (privacy, self.kind) {
            (Privacy::Public, _) | (Privacy::Default, ClassKind::Interface) => true,

            (Privacy::Protected, _)
            | (Privacy::Private, _)
            | (Privacy::Default, ClassKind::Class) => false,
        }
    }

    pub fn generics_scope(&self) -> GenericsScope<'_> {
        GenericsScope::Generics(&self.generics, &GenericsScope::Empty)
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Deserialize, Serialize)]
pub struct Generic {
    pub id: Id,
    pub extends: Vec<ClassRef>,
}

impl Generic {
    pub fn to_ident(&self, span: Span) -> Ident {
        self.id.to_ident(span)
    }
}

impl std::fmt::Display for Generic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)?;
        if let Some((e0, e1)) = self.extends.split_first() {
            write!(f, " extends {e0}")?;
            for ei in e1 {
                write!(f, " & {ei}")?;
            }
        }
        Ok(())
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum ClassKind {
    Class,
    Interface,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Flags {
    pub privacy: Privacy,
    pub is_final: bool,
    pub is_synchronized: bool,
    pub is_native: bool,
    pub is_abstract: bool,
    pub is_static: bool,
    pub is_default: bool,
    pub is_transient: bool,
    pub is_volatile: bool,
}

impl Flags {
    pub fn new(p: Privacy) -> Self {
        Flags {
            privacy: p,
            is_final: false,
            is_synchronized: false,
            is_native: false,
            is_abstract: false,
            is_static: false,
            is_default: false,
            is_transient: false,
            is_volatile: false,
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Privacy {
    Public,
    Protected,
    Private,

    /// NB: The default privacy depends on context.
    /// In a class, it is package.
    /// In an interface, it is public.
    Default,
}

impl std::fmt::Display for Privacy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Privacy::Public => write!(f, "`public`"),
            Privacy::Protected => write!(f, "`protected`"),
            Privacy::Private => write!(f, "`private`"),
            Privacy::Default => write!(f, "default privacy"),
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum MemberFunction {
    Constructor(Constructor),
    Method(Method),
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Serialize, Deserialize)]
pub struct Constructor {
    pub flags: Flags,
    pub generics: Vec<Generic>,
    pub argument_tys: Vec<Type>,
    pub throws: Vec<ClassRef>,
}

impl Constructor {
    pub fn to_method_sig(&self, class: &JavapClassInfo) -> MethodSig {
        MethodSig {
            name: class.name.class_name().clone(),
            generics: self.generics.clone(),
            argument_tys: self.argument_tys.clone(),
        }
    }

    /// Returns the JVM descriptor script for the constructor.
    ///
    /// # Parameters
    ///
    /// * `ctx` is the generics scope of the class.
    pub fn descriptor(&self, ctx: &GenericsScope<'_>) -> String {
        let ctx = &ctx.nest(&self.generics);
        format!(
            "({})V",
            self.argument_tys
                .iter()
                .map(|a| a.descriptor(ctx))
                .collect::<String>()
        )
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub flags: Flags,
    pub name: Id,
    pub ty: Type,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Serialize, Deserialize)]
pub struct Method {
    pub flags: Flags,
    pub name: Id,
    pub generics: Vec<Generic>,
    pub argument_tys: Vec<Type>,
    pub return_ty: Option<Type>,
    pub throws: Vec<ClassRef>,
}

impl Method {
    pub fn to_method_sig(&self) -> MethodSig {
        MethodSig {
            name: self.name.clone(),
            generics: self.generics.clone(),
            argument_tys: self.argument_tys.clone(),
        }
    }

    /// Returns the JVM descriptor for the method.
    ///
    /// # Parameters
    ///
    /// * `ctx` is the generics scope of the class.
    pub fn descriptor(&self, ctx: &GenericsScope<'_>) -> String {
        let ctx = &ctx.nest(&self.generics);
        Self::descriptor_from_types(ctx, &self.argument_tys, &self.return_ty)
    }

    pub fn descriptor_from_types(
        ctx: &GenericsScope<'_>,
        argument_tys: &[Type],
        return_ty: &Option<Type>,
    ) -> String {
        format!(
            "({}){}",
            argument_tys
                .iter()
                .map(|a| a.descriptor(ctx))
                .collect::<String>(),
            return_ty
                .as_ref()
                .map(|r| r.descriptor(ctx))
                .unwrap_or_else(|| format!("V")),
        )
    }
}

/// Signature of a single method in a class;
/// identifies the method precisely enough
/// to select from one of many overloaded methods.
#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct MethodSig {
    pub name: Id,
    pub generics: Vec<Generic>,
    pub argument_tys: Vec<Type>,
}

impl std::fmt::Display for MethodSig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((generic_id0, generic_ids)) = self.generics.split_first() {
            write!(f, "<{generic_id0}")?;
            for id in generic_ids {
                write!(f, ", {id}")?;
            }
            write!(f, "> ")?;
        }
        write!(f, "{}(", self.name)?;
        if let Some((ty0, tys)) = self.argument_tys.split_first() {
            write!(f, "{ty0}")?;
            for ty in tys {
                write!(f, ", {ty}")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Deserialize, Serialize)]
pub struct ClassRef {
    pub name: DotId,
    pub generics: Vec<RefType>,
}

impl std::fmt::Display for ClassRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some((ty0, tys)) = self.generics.split_first() {
            write!(f, "<{ty0}")?;
            for ty in tys {
                write!(f, ", {ty}")?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Serialize, Deserialize)]
pub enum Type {
    Ref(RefType),
    Scalar(ScalarType),
    Repeat(Arc<Type>),
}

impl From<ClassRef> for Type {
    fn from(value: ClassRef) -> Self {
        Type::Ref(RefType::Class(value))
    }
}

impl From<ScalarType> for Type {
    fn from(value: ScalarType) -> Self {
        Type::Scalar(value)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Ref(t) => write!(f, "{t}"),
            Type::Scalar(t) => write!(f, "{t}"),
            Type::Repeat(t) => write!(f, "{t}..."),
        }
    }
}

impl Type {
    pub fn is_scalar(&self) -> bool {
        match self {
            Type::Scalar(_) => true,
            Type::Ref(_) | Type::Repeat(_) => false,
        }
    }

    /// Convert a potentially repeating type to a non-repeating one.
    /// Types like `T...` become an array `T[]`.
    pub fn to_non_repeating(&self) -> NonRepeatingType {
        match self {
            Type::Ref(t) => NonRepeatingType::Ref(t.clone()),
            Type::Scalar(t) => NonRepeatingType::Scalar(t.clone()),
            Type::Repeat(t) => NonRepeatingType::Ref(RefType::Array(t.clone())),
        }
    }

    /// Returns the JVM descriptor for this type, suitable for embedding a method descriptor.
    ///
    /// # Parameters
    ///
    /// * `ctx` is the generics scope where the type appears.
    pub fn descriptor(&self, ctx: &GenericsScope<'_>) -> String {
        self.to_non_repeating().descriptor(ctx)
    }
}

/// Track generics currently in scope
///
/// In order to resolve descriptors for methods containing `<X extends Y>`, we need to know how `T` was declared.
pub enum GenericsScope<'a> {
    Empty,
    Generics(&'a [Generic], &'a GenericsScope<'a>),
}

impl<'a> GenericsScope<'a> {
    /// Find a generic within this scope by name
    fn find(&self, ty: &Id) -> Option<&Generic> {
        match self {
            GenericsScope::Empty => None,
            GenericsScope::Generics(g, inner) => g.iter().find(|g| &g.id == ty).or(inner.find(ty)),
        }
    }

    /// Add an additional layer to this scope (e.g. combining generics from a class with generics from a method)
    ///
    /// The newly provided generics are higher priority than the inner generics (but
    /// I don't think we can have namespace collisions here in Java anyway)
    fn nest(&'a self, generics: &'a [Generic]) -> GenericsScope<'a> {
        GenericsScope::Generics(generics, self)
    }
}

/// A variant of type
#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum NonRepeatingType {
    Ref(RefType),
    Scalar(ScalarType),
}

impl NonRepeatingType {
    /// Returns the JVM descriptor for this type, suitable for embedding a method descriptor.
    ///
    /// # Parameters
    ///
    /// * `ctx` is the generics scope where the type appears.
    pub fn descriptor(&self, ctx: &GenericsScope<'_>) -> String {
        match self {
            NonRepeatingType::Ref(r) => match r {
                RefType::Class(c) => format!("L{};", c.name.to_jni_name()),
                RefType::Array(r) => format!("[{}", r.descriptor(ctx)),

                RefType::TypeParameter(id) => {
                    let generic = ctx.find(id).expect("generic did not exist.");
                    match generic.extends.get(0) {
                        Some(c) => format!("L{};", c.name.to_jni_name()),
                        _ => format!("Ljava/lang/Object;"),
                    }
                }
                RefType::Extends(_) | RefType::Super(_) | RefType::Wildcard => {
                    format!("Ljava/lang/Object;")
                }
            },
            NonRepeatingType::Scalar(s) => s.descriptor().to_string(),
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Serialize, Deserialize)]
pub enum RefType {
    Class(ClassRef),
    Array(Arc<Type>),
    TypeParameter(Id),
    Extends(Arc<RefType>),
    Super(Arc<RefType>),
    Wildcard,
}

impl std::fmt::Display for RefType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefType::Class(c) => write!(f, "{c}"),
            RefType::Array(e) => write!(f, "{e}[]"),
            RefType::TypeParameter(id) => write!(f, "{id}"),
            RefType::Extends(t) => write!(f, "? extends {t}"),
            RefType::Super(t) => write!(f, "? super {t}"),
            RefType::Wildcard => write!(f, "?"),
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug, Serialize, Deserialize)]
pub enum ScalarType {
    Int,
    Long,
    Short,
    Byte,
    F64,
    F32,
    Boolean,
    Char,
}

impl std::fmt::Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::Int => write!(f, "int"),
            ScalarType::Long => write!(f, "long"),
            ScalarType::Short => write!(f, "long"),
            ScalarType::Byte => write!(f, "byte"),
            ScalarType::F64 => write!(f, "double"),
            ScalarType::F32 => write!(f, "float"),
            ScalarType::Boolean => write!(f, "boolean"),
            ScalarType::Char => write!(f, "char"),
        }
    }
}

impl ScalarType {
    pub fn to_tokens(&self, span: Span) -> TokenStream {
        match self {
            ScalarType::Char => quote_spanned!(span => u16),
            ScalarType::Int => quote_spanned!(span => i32),
            ScalarType::Long => quote_spanned!(span => i64),
            ScalarType::Short => quote_spanned!(span => i16),
            ScalarType::Byte => quote_spanned!(span => i8),
            ScalarType::F64 => quote_spanned!(span => f64),
            ScalarType::F32 => quote_spanned!(span => f32),
            ScalarType::Boolean => quote_spanned!(span => bool),
        }
    }

    pub fn descriptor(&self) -> &'static str {
        match self {
            ScalarType::Int => "I",
            ScalarType::Long => "J",
            ScalarType::Short => "S",
            ScalarType::Byte => "B",
            ScalarType::F64 => "D",
            ScalarType::F32 => "F",
            ScalarType::Boolean => "Z",
            ScalarType::Char => "C",
        }
    }
}

/// A single identifier
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Clone, Debug, Serialize, Deserialize)]
pub struct Id {
    pub data: String,
}

impl std::ops::Deref for Id {
    type Target = str;

    fn deref(&self) -> &str {
        &self.data
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id { data: value }
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id {
            data: value.to_owned(),
        }
    }
}

impl From<&Ident> for Id {
    fn from(value: &Ident) -> Self {
        Id {
            data: value.to_string(),
        }
    }
}

impl Id {
    pub fn dot(self, s: &str) -> DotId {
        DotId::from(self).dot(s)
    }

    pub fn to_ident(&self, span: Span) -> Ident {
        let data = self.data.replace("$", "__");
        Ident::new(&data, span)
    }

    pub fn to_snake_case(&self) -> Self {
        Self {
            data: self.data.to_snake_case(),
        }
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

/// A dotted identifier
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct DotId {
    /// Dotted components. Invariant: len >= 1.
    ids: Vec<Id>,
}

impl Serialize for DotId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.with_sep("."))
    }
}

impl<'de> Deserialize<'de> for DotId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(DotId::parse(&s))
    }
}

impl From<Id> for DotId {
    fn from(value: Id) -> Self {
        DotId { ids: vec![value] }
    }
}

impl From<&Id> for DotId {
    fn from(value: &Id) -> Self {
        DotId {
            ids: vec![value.clone()],
        }
    }
}

impl FromIterator<Id> for DotId {
    fn from_iter<T: IntoIterator<Item = Id>>(iter: T) -> Self {
        let ids: Vec<Id> = iter.into_iter().collect();
        assert!(ids.len() >= 1);
        DotId { ids }
    }
}

/// Handles java path resolution for codegen: how to write a `java.*` class
/// name.
///
/// A `java.*` name can mean two things: a class from the current java_package!
/// macro, or a class from the duchess prelude. The resolver checks the map of
/// java classes defined in the current `java_package!` macro and writes local
/// names relatively (`java::io::Foo`) and everything else as an absolute
/// prelude path (`duchess::java::lang::String`).
///
/// For example, if the java_package! macro declares
/// `java.io.ByteArrayOutputStream`, the Rust struct for it is generated into
/// the local `mod java`, under `io`, at the user's call site, and the prelude
/// has no such class, so writing `duchess::java::io::ByteArrayOutputStream`
/// would not compile. Or worse, it could point somewhere unintended.
///
/// The generated modules never import the prelude, so a plain `java::...` name
/// always means the local `mod java`. An absolute path is the only way to reach
/// the prelude.
///
/// Note: the derive macros (`ToRust`, `ToJava`) do not use this type. They
/// expand at the user's call site with no local class map, so they always
/// mean the prelude for `java.*` names (see `rust_class_name` in the macro
/// crate). If you change the rule here, check whether that helper needs
/// the same change.
#[derive(Clone, Copy, Debug)]
pub struct JavaPathResolver<'a> {
    local_classes: Option<&'a BTreeMap<DotId, Arc<ClassInfo>>>,
}

impl<'a> JavaPathResolver<'a> {
    /// Resolver for an invocation from its class map.
    pub fn for_root_map(root_map: &'a RootMap) -> Self {
        JavaPathResolver {
            local_classes: Some(&root_map.classes),
        }
    }

    /// Resolver with no local classes. Any `java.*` name resolves to the
    /// prelude.
    pub fn empty() -> Self {
        JavaPathResolver {
            local_classes: None,
        }
    }

    /// Returns the Rust path for the class `name`.
    pub fn rust_name(self, name: &DotId, span: Span) -> TokenStream {
        let is_local = self
            .local_classes
            .is_some_and(|local_classes| local_classes.contains_key(name));
        if name.is_java_path() && !is_local {
            name.to_duchess_prelude_name(span)
        } else {
            name.to_module_name(span)
        }
    }
}

impl DotId {
    pub fn new(package: &[Id], class: &Id) -> Self {
        DotId {
            ids: package
                .iter()
                .chain(std::iter::once(class))
                .cloned()
                .collect(),
        }
    }

    pub fn duchess() -> Self {
        Self::parse("duchess")
    }

    pub fn object() -> Self {
        Self::parse("java.lang.Object")
    }

    pub fn exception() -> Self {
        Self::parse("java.lang.Exception")
    }

    pub fn runtime_exception() -> Self {
        Self::parse("java.lang.RuntimeException")
    }

    pub fn throwable() -> Self {
        Self::parse("java.lang.Throwable")
    }

    pub fn parse(s: impl AsRef<str>) -> DotId {
        let s: &str = s.as_ref();
        let ids: Vec<Id> = s.split(".").map(Id::from).collect();
        assert!(ids.len() > 1, "bad input to DotId::parse: {s:?}");
        DotId { ids }
    }

    pub fn dot(mut self, s: &str) -> DotId {
        self.ids.push(Id::from(s));
        self
    }

    pub fn is_class(&self, s: &Id) -> bool {
        self.split().1 == s
    }

    /// returns the class name in JNI format with _'s escaped with _1
    /// https://docs.oracle.com/en/java/javase/17/docs/specs/jni/design.html
    pub fn to_jni_class_name(&self) -> Id {
        self.split().1.data.replace("_", "_1").into()
    }

    pub fn class_name(&self) -> &Id {
        self.split().1
    }

    /// returns the package in JNI format with _'s escaped with _1
    /// https://docs.oracle.com/en/java/javase/17/docs/specs/jni/design.html
    pub fn to_jni_package(&self) -> String {
        self.split()
            .0
            .iter()
            .map(|id| id.data.replace("_", "_1"))
            .collect::<Vec<_>>()
            .join("_")
    }

    /// Split and return the (package name, class name) pair.
    pub fn split(&self) -> (&[Id], &Id) {
        let (name, package) = self.ids.split_last().unwrap();
        (package, name)
    }

    /// Create a string-ified version of this name with `sep` in between each component.
    fn with_sep(&self, sep: &str) -> String {
        self.ids
            .iter()
            .map(|id| &id[..])
            .collect::<Vec<_>>()
            .join(sep)
    }

    /// Returns a name like `java/lang/Object`
    pub fn to_jni_name(&self) -> String {
        self.with_sep("/")
    }

    /// Returns a name like `java$lang$Object`
    pub fn to_dollar_name(&self) -> String {
        self.with_sep("$")
    }

    /// Returns a token stream like `java::lang::Object`
    pub fn to_module_name(&self, span: Span) -> TokenStream {
        let (package_names, struct_name) = self.split();
        let struct_ident = struct_name.to_ident(span);
        let package_idents: Vec<Ident> = package_names.iter().map(|n| n.to_ident(span)).collect();
        quote_spanned!(span => #(#package_idents ::)* #struct_ident)
    }

    /// True for `java.*` paths: paths whose first segment is `java`, like
    /// `java.io.Foo` but not `com.foo.java.Bar`. Only such paths can mean the
    /// local `mod java` (see `JavaPathResolver`).
    pub fn is_java_path(&self) -> bool {
        let (package_names, _) = self.split();
        matches!(package_names.first(), Some(id) if &id[..] == "java")
    }

    /// Returns an absolute path into the duchess `java` prelude, like
    /// `duchess::java::lang::Object`. There is no leading `::`, so the path
    /// also works when duchess builds itself, where `java.rs` aliases the
    /// current crate as `duchess`.
    pub fn to_duchess_prelude_name(&self, span: Span) -> TokenStream {
        let (package_names, struct_name) = self.split();
        let package_idents: Vec<Ident> = package_names.iter().map(|n| n.to_ident(span)).collect();
        let struct_ident = struct_name.to_ident(span);
        quote_spanned!(span => duchess :: #(#package_idents ::)* #struct_ident)
    }
}

impl std::ops::Deref for DotId {
    type Target = [Id];

    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

impl std::fmt::Display for DotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (package, class) = self.split();
        for id in package {
            write!(f, "{id}.")?;
        }
        write!(f, "{class}")?;
        Ok(())
    }
}

mod from_syn;
mod javap;

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, sync::Arc};

    use proc_macro2::Span;

    use super::*;
    use crate::upcasts::Upcasts;

    #[test]
    fn is_java_path_checks_first_segment() {
        for name in ["java.io.Foo", "java.lang.String", "java.Foo"] {
            assert!(DotId::parse(name).is_java_path(), "{name}");
        }
        for name in ["org.apache.Foo", "com.foo.java.Bar", "scala.Option"] {
            assert!(!DotId::parse(name).is_java_path(), "{name}");
        }
    }

    #[test]
    fn duchess_prelude_name_is_absolute() {
        let tokens = DotId::parse("java.lang.String").to_duchess_prelude_name(Span::call_site());
        assert_eq!(tokens.to_string(), "duchess :: java :: lang :: String");
    }

    /// Builds a `RootMap` from dotted package paths, nesting packages the
    /// same way the parser does, so `RootMap.subpackages` holds only first
    /// segments.
    fn root_with_packages(paths: &[&str]) -> RootMap {
        fn insert(map: &mut BTreeMap<Id, SpannedPackageInfo>, segments: &[&str]) {
            let Some((head, tail)) = segments.split_first() else {
                return;
            };
            let node = map
                .entry(Id::from(*head))
                .or_insert_with(|| SpannedPackageInfo {
                    name: Id::from(*head),
                    span: Span::call_site(),
                    subpackages: BTreeMap::new(),
                    classes: Vec::new(),
                });
            insert(&mut node.subpackages, tail);
        }

        let mut root = RootMap {
            subpackages: BTreeMap::new(),
            classes: BTreeMap::new(),
            upcasts: Upcasts::default(),
        };
        for path in paths {
            insert(&mut root.subpackages, &path.split('.').collect::<Vec<_>>());
        }
        root
    }

    /// A bare class entry, just enough to put a name in `RootMap.classes`.
    /// Only the key is ever read; the contents do not matter.
    fn dummy_class(name: DotId) -> std::sync::Arc<super::ClassInfo> {
        Arc::new(ClassInfo {
            span: Span::call_site(),
            flags: Flags::new(Privacy::Public),
            name,
            kind: ClassKind::Class,
            generics: Vec::new(),
            extends: Vec::new(),
            implements: Vec::new(),
            constructors: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
        })
    }

    #[test]
    fn resolver_without_local_classes_uses_absolute_prelude_paths() {
        use super::JavaPathResolver;

        let span = Span::call_site();
        let resolver = JavaPathResolver::empty();

        assert_eq!(
            resolver
                .rust_name(&DotId::parse("java.lang.String"), span)
                .to_string(),
            "duchess :: java :: lang :: String",
        );
        assert_eq!(
            resolver
                .rust_name(&DotId::parse("com.foo.Bar"), span)
                .to_string(),
            "com :: foo :: Bar",
        );
    }

    #[test]
    fn resolver_keeps_local_classes_relative() {
        use super::JavaPathResolver;

        let span = Span::call_site();
        let local = DotId::parse("java.io.Foo");
        let external = DotId::parse("java.lang.String");

        let mut root = root_with_packages(&["java.io"]);
        root.classes
            .insert(local.clone(), dummy_class(local.clone()));
        let resolver = JavaPathResolver::for_root_map(&root);

        assert_eq!(
            resolver.rust_name(&local, span).to_string(),
            "java :: io :: Foo",
        );
        assert_eq!(
            resolver.rust_name(&external, span).to_string(),
            "duchess :: java :: lang :: String",
        );
    }
}
