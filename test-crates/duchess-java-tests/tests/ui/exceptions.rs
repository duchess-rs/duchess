//@run
use duchess::java;
use duchess::prelude::*;
use duchess::Global;

duchess::java_package! {
    package exceptions;

    public class ThrowExceptions { * }
}

pub fn main() -> duchess::GlobalResult<()> {
    check_exceptions()?;
    check_static_fields()?;

    Ok(())
}

fn check_static_fields() -> duchess::GlobalResult<()> {
    let result = exceptions::ThrowExceptions::get_static_string_not_null()
        .global()
        .to_rust()
        .execute()?
        .unwrap();
    assert_eq!("notnull", result);
    Ok(())
}

fn check_exceptions() -> duchess::GlobalResult<()> {
    let thrower = exceptions::ThrowExceptions::new().global().execute()?;

    let result = thrower
        .throw_runtime()
        .execute()
        .expect_err("method throws an exception");
    assert!(matches!(result, duchess::Error::Thrown(_)));
    let error_message = format!("{}", result);
    assert!(
        error_message.contains("java.lang.RuntimeException: something has gone horribly wrong"),
        "{}",
        error_message
    );

    let error = thrower
        .null_object()
        .to_string()
        .global()
        .to_rust()
        .execute()
        .expect_err("returns a null pointer");

    assert!(matches!(error, duchess::Error::NullDeref));

    let misbehaved_exception = thrower
        .throw_exception_with_crashing_message()
        .execute()
        .expect_err("method doubly throws an exception");
    assert!(format!("{:?}", misbehaved_exception).contains("My exception threw an exception"));

    Ok(())
}
