//@check-pass
use duchess::prelude::*;

duchess::java_package! {
    package take_null;

    public class TakeNull { * }
    public class TakeNullRecord { * }
}

#[derive(duchess::ToJava)]
#[java(take_null.TakeNullRecord)]
struct TakeNullRecord {
    field: Option<String>
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

    let take_null_record = TakeNullRecord {
        field: Some(String::from("mystring")),
    };

    let java = take_null_record.to_java().execute().unwrap().unwrap();
    let is_null = java.is_null().execute()?;
    assert!(!is_null);

    let take_null_record = TakeNullRecord {
        field: None,
    };

    let java = take_null_record.to_java().execute().unwrap().unwrap();
    let is_null = java.is_null().execute()?;
    assert!(is_null);

    Ok(())
}
