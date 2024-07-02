//@run
use duchess::{java, prelude::*};

// Declare the java class that includes a native method
duchess::java_package! {
    package native_greeting;

    public class native_greeting.Native {
        public native_greeting.Native();
        public java.lang.String greet(java.lang.String);
        native java.lang.String baseGreeting(java.lang.String);
    }
}

// Implement the native method with a Rust function
#[duchess::java_function(native_greeting.Native::baseGreeting)]
fn base_greeting(
    _this: &native_greeting::Native,
    name: &java::lang::String,
) -> duchess::Result<String> {
    let name: String = name.execute()?;
    Ok(format!("Hello, {name}"))
}

fn main() -> duchess::Result<()> {
    // When creating the JVM, link the native method
    // by calling `link`.
    duchess::Jvm::builder()
        .link(base_greeting::java_fn())
        .try_launch()?;

    // Call the `greet` method in Java; this will invoke
    // the native `baseGreeting` method, which will call into
    // the Rust function above.
    let n: String = native_greeting::Native::new()
        .greet("Ferris")
        .assert_not_null()
        .execute()
        .unwrap();

    // Final result:
    //
    // * Java function calls Rust with "Ferris" as argument
    // * Rust function returns "Hello, Ferris"
    // * Java function appends ", from Java"
    assert_eq!(n, "Hello, Ferris, from Java");

    Ok(())
}
