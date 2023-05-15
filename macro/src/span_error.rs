use proc_macro2::{Literal, Span, TokenStream};
use quote::quote_spanned;

#[derive(Debug)]
pub struct SpanError {
    pub span: Span,
    pub message: String,
}

impl SpanError {
    pub fn into_tokens(self) -> TokenStream {
        let SpanError { span, message } = self;
        let message = Literal::string(&message);
        quote_spanned! { span => compile_error! { #message } }
    }
}

impl From<SpanError> for syn::Error {
    fn from(value: SpanError) -> Self {
        syn::Error::new(value.span, value.message)
    }
}
