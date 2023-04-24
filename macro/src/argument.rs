use proc_macro2::Span;

use crate::{
    class_info::{ClassDecl, Id},
    parse::{Parse, Parser},
    span_error::SpanError,
};

pub struct DuchessDeclaration {
    pub packages: Vec<JavaPackage>,
}

impl Parse for DuchessDeclaration {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let packages = JavaPackage::parse_many(p)?;
        Ok(Some(DuchessDeclaration { packages }))
    }

    fn description() -> String {
        format!("list of classes whose methods you would like to call (e.g., `java.lang.Object`)")
    }
}

pub struct JavaPackage {
    pub package_name: JavaPath,
    pub classes: Vec<ClassDecl>,
}

impl Parse for JavaPackage {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let Some(()) = p.eat_keyword("package") else {
            return Ok(None);
        };

        let Some(package_name) = JavaPath::parse(p)? else {
            return Err(p.error("expected package name"));
        };

        let Some(_) = p.eat_punct(';') else {
            return Err(p.error("expected `;` after package name"));
        };

        let classes = ClassDecl::parse_many(p)?;

        Ok(Some(JavaPackage {
            package_name,
            classes,
        }))
    }

    fn description() -> String {
        format!("java package to reflect (e.g., `package foo; ...`)")
    }
}

pub struct JavaPath {
    pub ids: Vec<Ident>,
    pub span: Span,
}

impl Parse for JavaPath {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let Some(text) = Ident::parse(p)? else {
            return Ok(None);
        };

        let mut span = text.span;
        let mut ids = vec![text];

        while let Some(_) = p.eat_punct('.') {
            let Some(next) = Ident::parse(p)? else {
                return Err(SpanError { span: p.last_span().unwrap(), message: format!("expected identifier after `.`") });
            };
            span = span.join(next.span).unwrap_or(span);
            ids.push(next);
        }

        Ok(Some(JavaPath { ids, span }))
    }

    fn description() -> String {
        format!("java class name (e.g., `java.lang.Object`)")
    }
}

impl std::fmt::Display for JavaPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((id0, ids)) = self.ids.split_first() {
            write!(f, "{}", id0)?;
            for id in ids {
                write!(f, ".{}", id)?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

pub struct Ident {
    pub text: String,
    pub span: Span,
}

impl Ident {
    pub fn to_id(&self) -> Id {
        Id::from(&self.text[..])
    }
}

impl Parse for Ident {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let Some(text) = p.eat_ident() else {
            return Ok(None);
        };

        Ok(Some(Ident {
            text,
            span: p.last_span().unwrap(),
        }))
    }

    fn description() -> String {
        format!("Java identifier")
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.text)
    }
}
