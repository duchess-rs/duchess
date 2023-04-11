use std::sync::Arc;

use proc_macro2::Span;

use crate::{parse::Parse, span_error::SpanError};

pub struct SpannedClassInfo {
    pub span: Span,
    pub info: ClassInfo,
}

impl SpannedClassInfo {
    pub fn parse(t: &str, span: Span) -> Result<Self, SpanError> {
        match javap::parse_class_info(&t) {
            Ok(info) => Ok(SpannedClassInfo { span, info }),
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
        let r = Self::parse(&t, span)?;
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
    pub generics: Vec<Id>,
    pub extends: Option<ClassRef>,
    pub implements: Vec<ClassRef>,
    pub constructors: Vec<Constructor>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct Flags {
    pub privacy: Privacy,
    pub is_final: bool,
    pub is_synchronized: bool,
    pub is_native: bool,
}

impl Flags {
    pub fn new(p: Privacy) -> Self {
        Flags {
            privacy: p,
            is_final: false,
            is_synchronized: false,
            is_native: false,
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
    pub args: Vec<Type>,
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

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub struct ClassRef {
    pub name: Id,
    pub generics: Vec<RefType>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub enum Type {
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
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

mod javap;
