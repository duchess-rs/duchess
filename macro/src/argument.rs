use proc_macro2::{Delimiter, Span};

use crate::{
    class_info::Id,
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
    pub classes: Vec<JavaClass>,
}

impl Parse for JavaPackage {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let Some(()) = p.eat_keyword("package") else {
            return Ok(None);
        };

        let Some(package_name) = JavaPath::parse(p)? else {
            return Err(p.error("expected package name"));
        };

        let Some(()) = p.eat_punct(';') else {
            return Err(p.error("expected `;` after package name"));
        };

        let classes = JavaClass::parse_many(p)?;

        Ok(Some(JavaPackage {
            package_name,
            classes,
        }))
    }

    fn description() -> String {
        format!("java package to reflect (e.g., `package foo; ...`)")
    }
}

pub struct JavaClass {
    pub class_span: Span,
    pub class_name: String,
    pub members: MemberListing,
}

impl Parse for JavaClass {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let Some(()) = p.eat_keyword("class") else {
            return Ok(None);
        };

        let Some(class_name) = p.eat_ident() else {
            return Err(p.error("expected class name"));
        };

        let class_span = p.last_span().unwrap();

        let Some(body) = p.eat_delimited(Delimiter::Brace) else {
            return Err(p.error("expected '{' after class name"));
        };

        let members = Parser::from(body).parse::<MemberListing>()?;

        Ok(Some(JavaClass {
            class_span,
            class_name,
            members,
        }))
    }

    fn description() -> String {
        format!("java class to reflect (e.g., `class Foo {{ * }}`)")
    }
}

pub enum MemberListing {
    All,
}

impl Parse for MemberListing {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        if let Some(()) = p.eat_punct('*') {
            return Ok(Some(MemberListing::All));
        }

        Ok(None)
    }

    fn description() -> String {
        format!("list of methods to accept, or `*` for all")
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

        while let Some(()) = p.eat_punct('.') {
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
