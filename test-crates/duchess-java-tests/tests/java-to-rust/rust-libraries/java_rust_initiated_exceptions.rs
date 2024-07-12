//@check-pass

use duchess::prelude::*;

duchess::java_package! {
    package java_rust_initiated_exceptions;

    public class java_rust_initiated_exceptions.JavaRustExceptions { * }
}

#[duchess::java_function(java_rust_initiated_exceptions.JavaRustExceptions::raiseNPE)]
fn raiseNPE(
    this: &java_rust_initiated_exceptions::JavaRustExceptions,
) -> duchess::Result<Java<java::lang::String>> {
    Err(duchess::Error::NullDeref)
}

#[duchess::java_function(java_rust_initiated_exceptions.JavaRustExceptions::raiseSliceTooLong)]
fn raiseSliceTooLong(
    this: &java_rust_initiated_exceptions::JavaRustExceptions,
) -> duchess::Result<Java<java::lang::String>> {
    Err(duchess::Error::SliceTooLong(5))
}

#[duchess::java_function(java_rust_initiated_exceptions.JavaRustExceptions::raiseJvmInternal)]
fn raiseJvmInternal(
    this: &java_rust_initiated_exceptions::JavaRustExceptions,
) -> duchess::Result<Java<java::lang::String>> {
    Err(duchess::Error::JvmInternal("JvmInternal".to_string()))
}
