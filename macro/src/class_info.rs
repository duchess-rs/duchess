use std::{collections::BTreeMap, sync::Arc};

use proc_macro2::{Ident, Span, TokenTree};

use crate::{
    argument::MemberListing,
    class_info::{self},
    parse::Parse,
    span_error::SpanError,
};

pub struct SpannedPackageInfo {
    pub name: Id,
    pub span: Span,
    pub subpackages: BTreeMap<Id, SpannedPackageInfo>,
    pub classes: Vec<SpannedClassInfo>,
}

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
    pub name: Id,
    pub kind: ClassKind,
    pub generics: Vec<Id>,
    pub extends: Option<ClassRef>,
    pub implements: Vec<ClassRef>,
    pub constructors: Vec<Constructor>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
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

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Field {
    pub flags: Flags,
    pub name: Id,
    pub ty: Type,
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

        let mut text: String = t0.to_string();
        let mut span = t0.span();

        if !is_semi(&t0) {
            while let Some(t1) = p.eat_token() {
                text.push_str(&t1.to_string());
                span = span.join(t1.span()).unwrap_or(span);
                if is_semi(&t1) {
                    break;
                }
            }
        }

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
        class.name == self.name
            && ctor.generics == self.generics
            && ctor.argument_tys == self.argument_tys
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct ClassRef {
    pub name: Id,
    pub generics: Vec<RefType>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum Type {
    Ref(RefType),
    Scalar(ScalarType),
    Repeat(Arc<Type>),
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

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum ScalarType {
    Int,
    Long,
    Short,
    Byte,
    F64,
    F32,
    Boolean,
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

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Id {
    pub data: Arc<String>,
}

impl std::ops::Deref for Id {
    type Target = String;

    fn deref(&self) -> &String {
        &self.data
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id {
            data: Arc::new(value),
        }
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id {
            data: Arc::new(value.to_owned()),
        }
    }
}

impl Id {
    pub fn dot(&self, s: &str) -> Id {
        Id::from(format!("{self}.{s}"))
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

mod javap;
