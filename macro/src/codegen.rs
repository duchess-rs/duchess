use crate::{argument::DuchessDeclaration, span_error::SpanError};
use proc_macro2::TokenStream;

impl DuchessDeclaration {
    pub fn into_tokens(mut self) -> Result<TokenStream, SpanError> {
        todo!()
    }
}
