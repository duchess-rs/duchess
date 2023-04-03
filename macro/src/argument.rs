use proc_macro2::Span;

use crate::{
    parse::{Parse, Parser},
    span_error::SpanError,
};

pub struct DuchessDeclaration {
    pub paths: Vec<JavaPath>,
}

impl Parse for DuchessDeclaration {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let Some(paths) = <Vec<JavaPath>>::parse(p)? else {
            return Ok(None);
        };

        Ok(Some(DuchessDeclaration { paths }))
    }

    fn description() -> String {
        format!("list of classes whose methods you would like to call (e.g., `java.lang.Object`)")
    }
}

pub struct JavaPath {
    pub text: String,
    pub span: Span,
}

impl Parse for JavaPath {
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let Some(mut text) = p.eat_ident() else {
            return Ok(None);
        };

        let span = p.last_span().unwrap();

        while let Some(()) = p.eat_punct('.') {
            let Some(next) = p.eat_ident() else {
                return Err(SpanError { span: p.last_span().unwrap(), message: format!("expected identifier after `.`") });
            };
            text.push_str(&next);
        }

        Ok(Some(JavaPath { text, span }))
    }

    fn description() -> String {
        format!("java class name (e.g., `java.lang.Object`)")
    }
}
