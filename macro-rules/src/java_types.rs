//! These macros take in a "java type description" and generate various Rust types to reflect it.
//! These java type descriptions are each a token tree and they have the following format:
//!
//! Java scalar types are a single identifier
//! * `int`
//! * `short`
//! * etc
//!
//! Java reference types are a `()`-token tree like:
//! * `(class[$path] $javaty*)`, e.g., `(class[java::util::Vector] (class[java::lang::String))` for `Vector<String>`
//! * `(array $javaty)`, e.g., `(array[(class[java::lang::String])])` for `String[]`
//! * `(generic $name)` to reference a generic (possible captured) type, e.g., `(generic[T])`

mod argument_impl_trait;
mod field_output_trait;
mod jni_call_fn;
mod jni_static_call_fn;
mod jni_static_field_get_fn;
mod output_trait;
mod output_type;
mod prepare_input;
mod rust_ty;
mod view_of_obj;
mod view_of_op;
