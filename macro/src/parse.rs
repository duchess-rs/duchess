use std::iter::Peekable;

use litrs::StringLit;
use proc_macro2::{Span, TokenStream, TokenTree};

use crate::span_error::SpanError;

pub struct Parser {
    tokens: Peekable<Box<dyn Iterator<Item = TokenTree>>>,
    last_span: Option<Span>,
}

impl From<TokenStream> for Parser {
    fn from(value: TokenStream) -> Self {
        let tokens: Box<dyn Iterator<Item = TokenTree>> = Box::new(value.into_iter());
        Parser {
            tokens: tokens.peekable(),
            last_span: None,
        }
    }
}

impl Parser {
    /// Top-level parse function that parses the input to the proc macro.
    pub fn parse<T: Parse>(mut self) -> Result<T, SpanError> {
        match T::parse(&mut self) {
            Ok(Some(t)) => {
                if let Some(s) = self.peek_span() {
                    return Err(SpanError {
                        span: s,
                        message: format!("extra input after the end of what was expected"),
                    });
                }
                Ok(t)
            }

            Err(e) => Err(e),

            Ok(None) => {
                let span = Span::call_site();
                return Err(SpanError {
                    span,
                    message: format!("expected a {}", T::description()),
                });
            }
        }
    }

    pub fn peek_token(&mut self) -> Option<&TokenTree> {
        self.tokens.peek()
    }

    pub fn peek_span(&mut self) -> Option<Span> {
        Some(self.peek_token()?.span())
    }

    pub fn last_span(&self) -> Option<Span> {
        self.last_span.clone()
    }

    pub fn eat_token(&mut self) -> Option<TokenTree> {
        let t = self.tokens.next()?;
        self.last_span = Some(t.span());
        Some(t)
    }

    pub fn eat_if<R>(&mut self, op: impl FnOnce(&TokenTree) -> Option<R>) -> Option<R> {
        let t = self.peek_token()?;
        let r = op(t)?;
        self.eat_token();
        Some(r)
    }

    pub fn eat_ident(&mut self) -> Option<String> {
        self.eat_if(|t| match t {
            TokenTree::Ident(i) => Some(i.to_string()),
            _ => None,
        })
    }

    pub fn eat_string_literal(&mut self) -> Option<String> {
        self.eat_if(|t| match StringLit::try_from(t) {
            Ok(v) => Some(v.into_value().into_owned()),
            Err(_) => None,
        })
    }

    pub fn eat_punct(&mut self, ch: char) -> Option<()> {
        self.eat_if(|t| match t {
            TokenTree::Punct(punct) if punct.as_char() == ch => Some(()),
            _ => None,
        })
    }
}

pub trait Parse: Sized {
    /// We assume an LL(1) grammar, so no need for backtracking.
    ///
    /// # Return value
    ///
    /// Err -- parse error after recognizing the start of a `Self`
    /// Ok(None) -- didn't recognize `Self` at this location
    /// Ok(Some(e)) -- successful parse of `Self`
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError>;

    /// Describes the thing we are parsing, for use in error messages.
    /// e.g. "java path".
    fn description() -> String;
}

impl<T> Parse for Vec<T>
where
    T: Parse,
{
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError> {
        let mut result = vec![];

        let Some(e0) = T::parse(p)? else {
            return Ok(None);
        };

        while let Some(()) = p.eat_punct(',') {
            let Some(e) = T::parse(p)? else {
                return Err(SpanError { span: p.last_span().unwrap(), message: format!("expected {} after `,`", T::description()) });
            };
            result.push(e);
        }

        // Permit trailing punctuation.
        let _ = p.eat_punct(',');

        Ok(Some(result))
    }

    fn description() -> String {
        format!("list of {}", T::description())
    }
}
