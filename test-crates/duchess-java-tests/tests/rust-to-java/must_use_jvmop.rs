use duchess::java;

#[deny(unused_must_use)]
fn main() -> duchess::Result<()> {
    let timestamp = java::util::Date::new();
    timestamp.set_time(0i64); //~ ERROR: unused implementer of `JvmOp` that must be used

    Ok(())
}
