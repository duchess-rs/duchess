//@run
use duchess::{java, prelude::*};

duchess::java_package! {
    package derives;
    class derives.OptionalFields { * }
}

#[derive(duchess::ToJava)]
#[java(derives.OptionalFields)]
struct OptionalFields {
    a: Option<String>,
    b: String,
}

pub fn main() -> duchess::Result<()> {
    let rust = OptionalFields {
        a: None,
        b: "hello".to_string(),
    };
    let java = rust.to_java().execute()?;
    Ok(())
}
