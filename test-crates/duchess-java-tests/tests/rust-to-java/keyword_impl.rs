//@run
use duchess::prelude::*;

duchess::java_package! {
    package keyword.impl;
    class keyword.impl.WithKeywordImpl { * }
}
fn main() -> duchess::Result<()> {
    let message: String = keyword::r#impl::WithKeywordImpl::get_message().assert_not_null().execute()?;
    assert_eq!(message, "impl keyword test");
    Ok(())
}
