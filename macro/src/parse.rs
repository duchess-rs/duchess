use std::iter::Peekable;

use litrs::StringLit;
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};

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

    /// Returns an error struct located at the last consumed token.
    pub fn error(&self, message: impl ToString) -> SpanError {
        SpanError {
            span: self.last_span().unwrap_or_else(|| Span::call_site()),
            message: message.to_string(),
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

    pub fn eat_token_if(&mut self, test: impl Fn(&TokenTree) -> bool) -> Option<TokenTree> {
        if test(self.peek_token()?) {
            self.eat_token()
        } else {
            None
        }
    }

    pub fn eat_delimited(&mut self, delimeter: Delimiter) -> Option<TokenStream> {
        self.eat_map(|t| match t {
            TokenTree::Group(g) if g.delimiter() == delimeter => Some(g.stream()),
            _ => None,
        })
    }

    pub fn eat_map<R>(&mut self, op: impl FnOnce(&TokenTree) -> Option<R>) -> Option<R> {
        let t = self.peek_token()?;
        let r = op(t)?;
        self.eat_token();
        Some(r)
    }

    pub fn eat_keyword(&mut self, kw: &str) -> Option<()> {
        assert!(KEYWORDS.contains(&kw));
        self.eat_map(|t| match t {
            TokenTree::Ident(i) => {
                let s = i.to_string();
                if s == kw {
                    Some(())
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    pub fn eat_ident(&mut self) -> Option<String> {
        self.eat_map(|t| match t {
            TokenTree::Ident(i) => {
                let s = i.to_string();
                if KEYWORDS.iter().any(|k| k == &s) {
                    None
                } else {
                    Some(i.to_string())
                }
            }
            _ => None,
        })
    }

    pub fn eat_string_literal(&mut self) -> Option<String> {
        self.eat_map(|t| match StringLit::try_from(t) {
            Ok(v) => Some(v.into_value().into_owned()),
            Err(_) => None,
        })
    }

    pub fn eat_punct(&mut self, ch: char) -> Option<Span> {
        self.eat_map(|t| match t {
            TokenTree::Punct(punct) if punct.as_char() == ch => Some(punct.span()),
            _ => None,
        })
    }
}

/// Utility class for accumulating tokens into a string, keeping a combined span
/// that contains (in theory) all the tokens accumulated. This class is a hack
/// used to bridge our LALRPOP parser, which operates on strings, with the user's
/// code. What we should really do is modify the LALRPOP parser to operate on tokens
/// directly, but that's for later.
pub struct TextAccum<'p> {
    text: String,
    span: Span,
    parser: &'p mut Parser,
}

impl<'p> TextAccum<'p> {
    pub fn new(parser: &'p mut Parser, t0: TokenTree) -> Self {
        Self {
            text: t0.to_string(),
            span: t0.span(),
            parser,
        }
    }

    /// Accumulate next token into the internal buffer and return it.
    pub fn accum(&mut self) -> Option<TokenTree> {
        self.accum_if(|_| true)
    }

    /// If the next token passes `test`, accumulates it into the internal buffer, and returns it.
    pub fn accum_if(&mut self, test: impl Fn(&TokenTree) -> bool) -> Option<TokenTree> {
        let t1 = self.parser.eat_token_if(test)?;
        self.text.push_str(&t1.to_string());
        self.span = self.span.join(t1.span()).unwrap_or(self.span);
        Some(t1)
    }

    /// Return the string we accumulated.
    pub fn into_accumulated_result(self) -> (String, Span) {
        (self.text, self.span)
    }
}

pub trait Parse: Sized {
    /// We assume an LL(1) grammar, so no need for backtracking.
    ///
    /// # Return value
    ///
    /// Err -- parse error after recognizing the start of a `Self`, may have consumed tokens
    /// Ok(None) -- didn't recognize `Self` at this location, didn't consume any tokens
    /// Ok(Some(e)) -- successful parse of `Self`
    fn parse(p: &mut Parser) -> Result<Option<Self>, SpanError>;

    /// parse any number of instances of self.
    fn parse_many(p: &mut Parser) -> Result<Vec<Self>, SpanError> {
        let mut result = vec![];

        while let Some(e) = Self::parse(p)? {
            result.push(e);
        }

        Ok(result)
    }

    /// Describes the thing we are parsing, for use in error messages.
    /// e.g. "java path".
    fn description() -> String;
}

/// Keywords not considered valid identifiers; subset of java keywords.
pub const KEYWORDS: &[&str] = &["package", "class"];
