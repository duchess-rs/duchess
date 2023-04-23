use std::{collections::BTreeMap, sync::Arc};

use proc_macro2::{Ident, Span, TokenTree};

use crate::{
    argument::{JavaClass, MemberListing},
    class_info::{self},
    parse::{Parse, TextAccum},
    span_error::SpanError,
};

/// Stores all the data about the classes/packages to be translated
/// as well as whatever we have learned from reflection.
#[derive(Debug)]
pub struct RootMap {
    pub subpackages: BTreeMap<Id, SpannedPackageInfo>,
}

impl RootMap {
    /// Finds the class with the given name (if present).
    pub fn find_class(&self, cn: &DotId) -> Option<&JavaClass> {
        let (package, class_name) = cn.split();
        let package_info = self.find_package(package)?;
        package_info.find_class(class_name)
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
        self.subpackages
            .values()
            .flat_map(|pkg| pkg.class_names(&[]))
            .collect()
    }
}

#[derive(Debug)]
pub struct SpannedPackageInfo {
    pub name: Id,
    pub span: Span,
    pub subpackages: BTreeMap<Id, SpannedPackageInfo>,
    pub classes: Vec<JavaClass>,
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
    pub fn find_class(&self, cn: &Id) -> Option<&JavaClass> {
        self.classes.iter().find(|c| c.class_name == *cn)
    }

    /// Find the names of all classes contained within self
    pub fn class_names(&self, parent_package: &[Id]) -> Vec<DotId> {
        // Name of this package
        let package_name: Vec<Id> = parent_package
            .iter()
            .chain(std::iter::once(&self.name))
            .cloned()
            .collect();

        let classes_from_subpackages = self
            .subpackages
            .values()
            .flat_map(|pkg| pkg.class_names(&package_name));

        let classes_from_this_package = self
            .classes
            .iter()
            .map(|c| DotId::new(&package_name, &c.class_name));

        classes_from_subpackages
            .chain(classes_from_this_package)
            .collect()
    }
}

#[derive(Debug)]
pub struct SpannedClassInfo {
    /// The complete class info loaded from javap
    pub info: ClassInfo,

    /// The span where user declared interest in this class
    pub span: Span,

    /// The listing of members user wants to include
    pub members: MemberListing,
}

impl SpannedClassInfo {
    pub fn parse(t: &str, span: Span, members: MemberListing) -> Result<Self, SpanError> {
        match javap::parse_class_info(&t) {
            Ok(info) => Ok(SpannedClassInfo {
                span,
                info,
                members,
            }),
            Err(message) => Err(SpanError { span, message }),
        }
    }
}

impl Parse for SpannedClassInfo {
    fn parse(p: &mut crate::parse::Parser) -> Result<Option<Self>, SpanError> {
        let Some(t) = p.eat_string_literal() else {
            return Ok(None);
        };
        let span = p.last_span().unwrap();
        let r = Self::parse(&t, span, MemberListing::all())?;
        Ok(Some(r))
    }

    fn description() -> String {
        format!("output from `javap -public -s`")
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct ClassInfo {
    pub flags: Flags,
    pub name: DotId,
    pub kind: ClassKind,
    pub generics: Vec<Id>,
    pub extends: Option<ClassRef>,
    pub implements: Vec<ClassRef>,
    pub constructors: Vec<Constructor>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

impl ClassInfo {
    pub fn parse(t: &str) -> Result<Self, String> {
        javap::parse_class_info(t)
    }

    pub fn this_ref(&self) -> ClassRef {
        ClassRef {
            name: self.name.clone(),
            generics: self
                .generics
                .iter()
                .map(|g| RefType::TypeParameter(g.clone()))
                .collect(),
        }
    }

    /// Constructors selected by the user for codegen
    pub fn selected_constructors<'m>(
        &'m self,
        members: &'m MemberListing,
    ) -> impl Iterator<Item = &'m Constructor> {
        self.constructors
            .iter()
            .filter(move |c| members.contains_constructor(self, c))
    }

    /// Methods selected by the user for codegen (note: some may be static)
    pub fn selected_methods<'m>(
        &'m self,
        members: &'m MemberListing,
    ) -> impl Iterator<Item = &'m Method> {
        self.methods
            .iter()
            .filter(move |m| members.contains_method(m))
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum ClassKind {
    Class,
    Interface,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Flags {
    pub privacy: Privacy,
    pub is_final: bool,
    pub is_synchronized: bool,
    pub is_native: bool,
    pub is_abstract: bool,
    pub is_static: bool,
    pub is_default: bool,
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
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum Privacy {
    Public,
    Protected,
    Package,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Constructor {
    pub flags: Flags,
    pub generics: Vec<Id>,
    pub argument_tys: Vec<Type>,
    pub throws: Vec<ClassRef>,
    pub descriptor: Descriptor,
}

impl Constructor {
    pub fn to_method_sig(&self, class: &ClassInfo) -> MethodSig {
        MethodSig {
            name: class.name.class_name().clone(),
            generics: self.generics.clone(),
            argument_tys: self.argument_tys.clone(),
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Field {
    pub flags: Flags,
    pub name: Id,
    pub ty: Type,
    pub descriptor: Descriptor,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Method {
    pub flags: Flags,
    pub name: Id,
    pub generics: Vec<Id>,
    pub argument_tys: Vec<Type>,
    pub return_ty: Option<Type>,
    pub throws: Vec<ClassRef>,
    pub descriptor: Descriptor,
}

impl Method {
    pub fn to_method_sig(&self) -> MethodSig {
        MethodSig {
            name: self.name.clone(),
            generics: self.generics.clone(),
            argument_tys: self.argument_tys.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpannedMethodSig {
    pub method_sig: MethodSig,
    pub span: Span,
}

impl Parse for SpannedMethodSig {
    fn parse(p: &mut crate::parse::Parser) -> Result<Option<Self>, SpanError> {
        // Parse an individual method. For this, we hackily consume all tokens until a `;`
        // and make a string out of it, then pass to the lalrpop parser.
        //
        // FIXME: clean this up.
        let Some(t0) = p.eat_token() else {
            return Ok(None);
        };

        if is_semi(&t0) {
            return Err(SpanError {
                span: t0.span(),
                message: format!("empty method signature"),
            });
        }

        let mut accum = TextAccum::new(p, t0);
        while let Some(t1) = accum.accum() {
            if is_semi(&t1) {
                break;
            }
        }

        let (text, span) = accum.into_accumulated_result();

        return match class_info::javap::parse_method_sig(&text) {
            Ok(ms) => Ok(Some(SpannedMethodSig {
                method_sig: ms,
                span,
            })),
            Err(message) => return Err(SpanError { span, message }),
        };

        fn is_semi(t: &TokenTree) -> bool {
            match t {
                TokenTree::Punct(p) => p.as_char() == ';',
                _ => false,
            }
        }
    }

    fn description() -> String {
        format!("java method signature")
    }
}

/// Signature of a single method in a class;
/// identifies the method precisely enough
/// to select from one of many overloaded methods.
#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct MethodSig {
    pub name: Id,
    pub generics: Vec<Id>,
    pub argument_tys: Vec<Type>,
}

impl MethodSig {
    pub fn matches(&self, m: &Method) -> bool {
        m.name == self.name && m.generics == self.generics && m.argument_tys == self.argument_tys
    }

    pub fn matches_constructor(&self, class: &ClassInfo, ctor: &Constructor) -> bool {
        class.name.is_class(&self.name)
            && ctor.generics == self.generics
            && ctor.argument_tys == self.argument_tys
    }
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

#[derive(Clone, Debug)]
pub struct SpannedClassRef {
    pub class_ref: ClassRef,
    pub span: Span,
}

impl Parse for SpannedClassRef {
    fn parse(p: &mut crate::parse::Parser) -> Result<Option<Self>, SpanError> {
        // Hackily consume a series of ident, `.` tokens until we either see
        // something else or we see a `<`. If we see a `<`, then consume tokens
        // until we see matching `>`.
        //
        // FIXME: clean this up.

        // Consume `foo.bar.baz`
        let Some(t0) = p.eat_token_if(is_part_of_java_path) else {
            return Ok(None);
        };
        let mut accum = TextAccum::new(p, t0);
        while let Some(_) = accum.accum_if(is_part_of_java_path) {}

        // Consume `<...>` by matching `<` and `>`.
        if let Some(_) = accum.accum_if(is_open_angle_bracket) {
            let mut counter = 1;
            while let Some(t) = accum.accum() {
                if is_punct(&t, '<') {
                    counter += 1;
                } else if is_punct(&t, '>') {
                    counter -= 1;
                    if counter == 0 {
                        break;
                    }
                }
            }
        }

        let (text, span) = accum.into_accumulated_result();

        return match class_info::javap::parse_class_ref(&text) {
            Ok(cr) => Ok(Some(SpannedClassRef {
                class_ref: cr,
                span,
            })),
            Err(message) => return Err(SpanError { span, message }),
        };

        fn is_part_of_java_path(t: &TokenTree) -> bool {
            match t {
                TokenTree::Punct(p) => p.as_char() == '.',
                TokenTree::Ident(_) => true,
                _ => false,
            }
        }

        fn is_open_angle_bracket(t: &TokenTree) -> bool {
            is_punct(t, '<')
        }

        fn is_punct(t: &TokenTree, ch: char) -> bool {
            match t {
                TokenTree::Punct(p) => p.as_char() == ch,
                _ => false,
            }
        }
    }

    fn description() -> String {
        format!("java method signature")
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
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

impl ClassRef {
    pub fn object() -> ClassRef {
        ClassRef {
            name: DotId::object(),
            generics: vec![],
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum Type {
    Ref(RefType),
    Scalar(ScalarType),
    Repeat(Arc<Type>),
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
    /// Convert a potentially repeating type to a non-repeating one.
    /// Types like `T...` become an array `T[]`.
    pub fn to_non_repeating(&self) -> NonRepeatingType {
        match self {
            Type::Ref(t) => NonRepeatingType::Ref(t.clone()),
            Type::Scalar(t) => NonRepeatingType::Scalar(t.clone()),
            Type::Repeat(t) => NonRepeatingType::Ref(RefType::Array(t.clone())),
        }
    }
}

/// A variant of type
#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum NonRepeatingType {
    Ref(RefType),
    Scalar(ScalarType),
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
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

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
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

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Descriptor {
    pub string: String,
}

impl From<&str> for Descriptor {
    fn from(value: &str) -> Self {
        Descriptor {
            string: value.to_string(),
        }
    }
}

/// A single identifier
#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Id {
    pub data: String,
}

impl std::ops::Deref for Id {
    type Target = String;

    fn deref(&self) -> &String {
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

impl Id {
    pub fn dot(self, s: &str) -> DotId {
        DotId::from(self).dot(s)
    }

    pub fn to_ident(&self, span: Span) -> Ident {
        Ident::new(&self.data, span)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

/// A dotted identifier
#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct DotId {
    /// Dotted components. Invariant: len >= 1.
    ids: Vec<Id>,
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

    pub fn class_name(&self) -> &Id {
        self.split().1
    }

    /// Split and return the (package name, class name) pair.
    pub fn split(&self) -> (&[Id], &Id) {
        let (name, package) = self.ids.split_last().unwrap();
        (package, name)
    }

    pub fn object() -> Self {
        Self::parse("java.lang.Object")
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

mod javap;
