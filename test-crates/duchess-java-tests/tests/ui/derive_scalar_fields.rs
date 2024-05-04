//@run
use duchess::{java, prelude::*};

#[derive(duchess::ToJava)]
#[java(java.time.Instant::ofEpochMilli)]
struct RustInstant {
    epoch_millis: i64,
}

pub fn main() -> duchess::Result<()> {
    let rust = RustInstant { epoch_millis: 42 };
    let java = rust.to_java().assert_not_null().global().execute()?;
    let and_back = java.to_epoch_milli().execute()?;
    assert_eq!(rust.epoch_millis, and_back);
    Ok(())
}
