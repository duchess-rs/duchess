use std::fmt::Display;

use lalrpop_util::{lalrpop_mod, lexer::Token};
use proc_macro2::Span;

use crate::span_error::SpanError;

use super::{ClassDecl, ClassInfo};

lalrpop_mod!(pub javap_parser, "/class_info/javap_parser.rs"); // synthesized by LALRPOP

pub(super) fn parse_class_decl(span: Span, input: &str) -> Result<ClassDecl, SpanError> {
    match javap_parser::ClassDeclParser::new().parse(span, input) {
        Ok(v) => Ok(v),
        Err(error) => Err(SpanError {
            span,
            message: format_lalrpop_error(input, error),
        }),
    }
}

pub(super) fn parse_class_info(span: Span, input: &str) -> Result<ClassInfo, SpanError> {
    match javap_parser::ClassInfoParser::new().parse(span, input) {
        Ok(v) => Ok(v),
        Err(error) => Err(SpanError {
            span,
            message: format_lalrpop_error(input, error),
        }),
    }
}

fn format_lalrpop_error(
    input: &str,
    error: lalrpop_util::ParseError<usize, Token<'_>, impl Display>,
) -> String {
    match error {
        lalrpop_util::ParseError::ExtraToken { token } => {
            format!("extra token at end of input (`{}`)", token.1)
        }
        lalrpop_util::ParseError::UnrecognizedEOF {
            location: _,
            expected,
        } => {
            format!("unexpected end of input, expected one of `{:?}`", expected)
        }
        lalrpop_util::ParseError::UnrecognizedToken {
            token: (start, _, end),
            expected,
        } => {
            let window_string = window_string(input, start, end);

            format!(
                "unexpected token `{}` at offset {}, expected one of `{:?}`",
                window_string, start, expected
            )
        }
        lalrpop_util::ParseError::InvalidToken { location } => {
            let ch_len = input[location..].chars().next().unwrap().len_utf8();
            let window_string = window_string(input, location, location + ch_len);
            format!("invalid token `{}` at offset {}", window_string, ch_len,)
        }
        lalrpop_util::ParseError::User { error } => format!("{}", error),
    }
}

fn window_string(input: &str, start: usize, end: usize) -> String {
    const WINDOW: usize = 22;

    let mut window_string = String::new();

    if start < WINDOW {
        window_string.push_str(&input[..start]);
    } else {
        window_string.push_str("... ");
        window_string.push_str(&input[start - WINDOW..start]);
    }

    window_string.push_str(" <<< ");
    window_string.push_str(&input[start..end]);
    window_string.push_str(" >>> ");

    let window_end = (end + WINDOW).min(input.len());
    window_string.push_str(&input[end..window_end]);
    if input.len() > window_end {
        window_string.push_str(" ...");
    }

    window_string
}
