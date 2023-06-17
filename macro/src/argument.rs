use proc_macro2::Span;

use crate::{
    class_info::{ClassDecl, ClassInfo, DotId, Id},
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

/// There are various points where the user must select
/// a method. In these cases, we permit them to either write
/// just a class name (in which case we search for (hopefully) at most one
/// such method), a class + method name, or a little mini class declaration
/// that includes the full details (which accommodates the case where it is
/// overloaded).
pub enum MethodSelector {
    /// User wrote `foo.bar.Baz`
    ClassName(JavaPath),

    /// User wrote `foo.bar.Baz::method`
    MethodName(JavaPath, Ident),

    /// User wrote `class Foo { ... }` with full details.
    /// This class should have at most one member.
    ClassInfo(ClassInfo),
}

impl MethodSelector {
    /// Span for things that refer to the method
    pub fn span(&self) -> Span {
        match self {
            MethodSelector::ClassName(jp) => jp.span,
            MethodSelector::MethodName(_, ident) => ident.span,
            MethodSelector::ClassInfo(ci) => ci.span,
        }
    }

    /// Span for things that refer to the class the method is in
    pub fn class_span(&self) -> Span {
        match self {
            MethodSelector::ClassName(jp) => jp.span,
            MethodSelector::MethodName(jp, _) => jp.span,
            MethodSelector::ClassInfo(ci) => ci.span,
        }
    }

    pub fn class_name(&self) -> DotId {
        match self {
            MethodSelector::ClassName(c) => c.to_dot_id(),
            MethodSelector::MethodName(c, _) => c.to_dot_id(),
            MethodSelector::ClassInfo(_) => todo!(),
        }
    }

    /// Returns the name of the method
    pub fn method_name(&self) -> String {
        match self {
            MethodSelector::ClassName(_) => self.class_name().split().1.to_string(),
            MethodSelector::MethodName(_, m) => m.to_string(),
            MethodSelector::ClassInfo(_) => todo!(),
        }
    }
}

impl Parse for MethodSelector {
    fn parse(p: &mut crate::parse::Parser) -> Result<Option<Self>, SpanError> {
        // Check for a `class` declaration
        if let Some(c) = ClassDecl::parse(p)? {
            return match c {
                ClassDecl::Reflected(r) => Err(SpanError {
                    span: r.span,
                    message: format!("expected a class with a single member, not `*`"),
                }),
                ClassDecl::Specified(c) => {
                    let members = c.constructors.len() + c.fields.len() + c.methods.len();
                    if members != 1 {
                        Err(SpanError {
                            span: c.span,
                            message: format!(
                                "expected a class with exactly one member, but {} members found",
                                members
                            ),
                        })
                    } else {
                        Ok(Some(MethodSelector::ClassInfo(c)))
                    }
                }
            };
        }

        // Otherwise we expect either `foo.bar.Baz` or `foo.bar.Baz::method`
        let Some(path) = JavaPath::parse(p)? else {
            return Ok(None);
        };

        if let Some(_) = p.eat_punct(':') {
            if let Some(_) = p.eat_punct(':') {
                if let Some(ident) = Ident::parse(p)? {
                    return Ok(Some(MethodSelector::MethodName(path, ident)));
                }
            }
            Err(SpanError {
                span: p.peek_span().unwrap_or(Span::call_site()),
                message: format!("expected method name after `::`"),
            })
        } else {
            Ok(Some(MethodSelector::ClassName(path)))
        }
    }

    fn description() -> String {
        format!("method selector, e.g. `java.package.Class`, `java.package.Class::method`, or full details")
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

impl JavaPath {
    pub fn to_dot_id(&self) -> DotId {
        self.ids.iter().map(|ident| ident.to_id()).collect()
    }
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
