use std::iter::FromIterator;
use std::{collections::BTreeMap, sync::Arc};

use crate::class_info::{ClassRef, Id, RefType, Type};

pub struct Substitution<'s> {
    map: BTreeMap<&'s Id, &'s RefType>,
}

impl<'s> FromIterator<(&'s Id, &'s RefType)> for Substitution<'s> {
    fn from_iter<T: IntoIterator<Item = (&'s Id, &'s RefType)>>(iter: T) -> Self {
        Substitution {
            map: iter.into_iter().collect(),
        }
    }
}

pub trait Substitute {
    fn substitute(&self, subst: &Substitution<'_>) -> Self;
}

impl Substitute for RefType {
    fn substitute(&self, subst: &Substitution<'_>) -> Self {
        match self {
            RefType::Class(c) => RefType::Class(c.substitute(subst)),
            RefType::Array(a) => RefType::Array(a.substitute(subst)),
            RefType::TypeParameter(id) => match subst.map.get(&id) {
                Some(v) => RefType::clone(v),
                None => self.clone(),
            },
            RefType::Extends(e) => RefType::Extends(e.substitute(subst)),
            RefType::Super(e) => RefType::Super(e.substitute(subst)),
            RefType::Wildcard => RefType::Wildcard,
        }
    }
}

impl Substitute for Type {
    fn substitute(&self, subst: &Substitution<'_>) -> Self {
        match self {
            Type::Ref(r) => Type::Ref(r.substitute(subst)),
            Type::Scalar(_) => self.clone(),
            Type::Repeat(r) => Type::Repeat(r.substitute(subst)),
        }
    }
}

impl Substitute for ClassRef {
    fn substitute(&self, subst: &Substitution<'_>) -> Self {
        let ClassRef { name, generics } = self;
        ClassRef {
            name: name.clone(),
            generics: generics.substitute(subst),
        }
    }
}

impl<F> Substitute for Vec<F>
where
    F: Substitute,
{
    fn substitute(&self, subst: &Substitution<'_>) -> Self {
        self.iter().map(|e| e.substitute(subst)).collect()
    }
}

impl<F> Substitute for Arc<F>
where
    F: Substitute,
{
    fn substitute(&self, subst: &Substitution<'_>) -> Self {
        Arc::new(F::substitute(self, subst))
    }
}
