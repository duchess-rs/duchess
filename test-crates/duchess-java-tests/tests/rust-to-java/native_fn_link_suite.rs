//@ run

use duchess::{java, prelude::*};

duchess::java_package! {
    package native_greeting;

    public class native_greeting.Native {
        public native_greeting.Native();
        public java.lang.String greet(java.lang.String);
        native java.lang.String baseGreeting(java.lang.String);
    }
}

#[duchess::java_function(native_greeting.Native::baseGreeting)]
fn base_greeting(
    _this: &native_greeting::Native,
    name: Option<&java::lang::String>,
) -> duchess::Result<String> {
    let name: String = name.assert_not_null().execute()?;
    Ok(format!("Hello, {name}"))
}

fn native_functions() -> Vec<duchess::JavaFunction> {
    vec![base_greeting::java_fn()]
}

fn main() -> duchess::Result<()> {
    duchess::Jvm::builder()
        .link(native_functions())
        .try_launch()?;

    let n: String = native_greeting::Native::new()
        .greet("Ferris")
        .assert_not_null()
        .execute()
        .unwrap();

    assert_eq!(n, "Hello, Ferris, from Java");

    Ok(())
}
