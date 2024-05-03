//@run
use duchess::prelude::*;
use duchess::Global;
use duchess::{java, Jvm};

duchess::java_package! {
    package exceptions;

    public class ThrowExceptions { * }
    public class DifferentException { * }
}

pub fn main() -> duchess::GlobalResult<()> {
    check_exceptions()?;
    check_static_fields()?;
    catch_exceptions()?;

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

fn catch_exceptions() -> duchess::GlobalResult<()> {
    // Note: perhaps an API issue, I was only able to get catch to work in a non-global context, hence `Jvm::with`
    Jvm::with(|jvm| {
        let thrower = exceptions::ThrowExceptions::new()
            .execute_with(jvm)
            .unwrap();

        let caught_exception = thrower
            .throw_runtime()
            .catch::<java::lang::RuntimeException>()
            .execute_with(jvm)
            .unwrap();
        assert!(
            // it matches the expected exception type so, outer is Ok, inner is err
            matches!(&caught_exception, Err(_)),
            "{:?}",
            caught_exception
        );

        let caught_exception = thrower
            .null_object()
            .catch::<java::lang::RuntimeException>()
            .execute_with(jvm)
            .unwrap()
            .expect("returns ok!");

        // This errors out because `try_extract_exception` calls `Jvm::with`.
        let caught_exception = thrower
            .throw_runtime()
            .catch::<exceptions::DifferentException>()
            .execute_with(jvm);
        assert!(matches!(caught_exception, Err(duchess::Error::Thrown(_))));

        // This is a reproduction of https://github.com/duchess-rs/duchess/issues/142
        assert_eq!(format!("{:?}", caught_exception), "Err(Java invocation threw: failed to get message: attempted to nest `Jvm::with` calls)");
        Ok(())
    })
    .unwrap();
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
