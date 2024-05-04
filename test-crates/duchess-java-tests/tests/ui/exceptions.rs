//@run
use duchess::prelude::*;
use duchess::Jvm;

duchess::java_package! {
    package exceptions;

    public class ThrowExceptions { * }
    public class DifferentException { * }
}

pub fn main() -> duchess::Result<()> {
    check_exceptions()?;
    check_static_fields()?;
    catch_exceptions()?;

    Ok(())
}

fn check_static_fields() -> duchess::Result<()> {
    let result: String = exceptions::ThrowExceptions::get_static_string_not_null()
        .execute()?
        .unwrap();
    assert_eq!("notnull", result);
    Ok(())
}

fn catch_exceptions() -> duchess::Result<()> {
    let thrower = exceptions::ThrowExceptions::new().execute().unwrap();

    let caught_exception = thrower
        .throw_runtime()
        .catch::<java::lang::RuntimeException>()
        .execute()
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
        .execute()
        .unwrap()
        .expect("returns ok!");

    // This errors out because `try_extract_exception` calls `Jvm::with`.
    let caught_exception = thrower
        .throw_runtime()
        .catch::<exceptions::DifferentException>()
        .execute();
    assert!(matches!(caught_exception, Err(duchess::Error::Thrown(_))));

    assert_eq!(
        format!("{:?}", caught_exception),
        "Err(Java invocation threw: java.lang.RuntimeException: something has gone horribly wrong)"
    );
    Ok(())
}

fn check_exceptions() -> duchess::Result<()> {
    let thrower = exceptions::ThrowExceptions::new().execute()?;

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
        .execute::<Option<String>>()
        .expect_err("returns a null pointer");

    assert!(matches!(error, duchess::Error::NullDeref));

    let misbehaved_exception = thrower
        .throw_exception_with_crashing_message()
        .execute()
        .expect_err("method doubly throws an exception");
    assert!(format!("{:?}", misbehaved_exception).contains("My exception threw an exception"));

    Ok(())
}
