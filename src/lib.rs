use proc_macro::TokenStream;

mod class_info;

/// The main duchess macro, used like so
///
/// ```rust
/// duchess! {
///     mod java {
///         java.lang.Object,
///         java.util.ArrayList { new, foo, bar },
///     }
/// }
/// ```
///
/// The
#[proc_macro]
pub fn duchess(input: TokenStream) -> TokenStream {
    input
}

struct DuchessDeclaration {}
