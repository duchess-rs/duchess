//@check-pass
use duchess::prelude::*;

duchess::java_package! {
    package take_null;

    public class TakeNull { * }
}

pub fn main() -> duchess::Result<()> {
    let take_null = take_null::TakeNull::new().execute()?;

    let is_null = take_null.take_null_object(duchess::Null).execute()?;
    assert!(is_null);

    let is_null = take_null
        .take_null_object(&None::<Java<java::lang::Object>>)
        .execute()?;
    assert!(is_null);

    let is_null = take_null.take_null_string(duchess::Null).execute()?;
    assert!(is_null);

    let is_null = take_null
        .take_null_string(&None::<Java<java::lang::String>>)
        .execute()?;
    assert!(is_null);

    Ok(())
}
