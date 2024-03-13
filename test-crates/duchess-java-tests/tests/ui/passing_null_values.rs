//@check-pass
use duchess::java;
use duchess::prelude::*;
use duchess::Global;

duchess::java_package! {
    package take_null;

    public class TakeNull { * }
}

pub fn main() -> duchess::GlobalResult<()> {
    let take_null = take_null::TakeNull::new().global().execute()?;

    let is_null = take_null.take_null_object(duchess::Null).execute()?;
    assert!(is_null);

    let is_null = take_null
        .take_null_object(&None::<Global<java::lang::Object>>)
        .execute()?;
    assert!(is_null);

    let is_null = take_null.take_null_string(duchess::Null).execute()?;
    assert!(is_null);

    Ok(())
}
