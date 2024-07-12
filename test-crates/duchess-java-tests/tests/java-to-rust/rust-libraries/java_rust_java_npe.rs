//@check-pass

use duchess::prelude::*;

duchess::java_package! {
    package java_rust_java_exception;

    public class java_rust_java_exception.JavaRustJavaNPE { * }
}

#[duchess::java_function(java_rust_java_exception.JavaRustJavaNPE::rustFunction)]
fn rust_function(
    this: &java_rust_java_exception::JavaRustJavaNPE,
) -> duchess::Result<Java<java::lang::String>> {
    Ok(this.java_function().assert_not_null().execute()?)
}
