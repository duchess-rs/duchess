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
fn base_greeting<'n>(
    _this: &native_greeting::Native,
    name: &'n java::lang::String,
) -> duchess::GlobalResult<duchess::Local<'n, java::lang::String>> {
    name.execute() //~ ERROR: mismatched types
}

fn main() -> duchess::GlobalResult<()> {
    duchess::Jvm::builder()
        .link(base_greeting::java_fn())
        .try_launch()?;

    let n: String = native_greeting::Native::new()
        .greet("Ferris")
        .assert_not_null()
        .to_rust()
        .unwrap();

    assert_eq!(n, "Ferris, from Java");

    Ok(())
}
