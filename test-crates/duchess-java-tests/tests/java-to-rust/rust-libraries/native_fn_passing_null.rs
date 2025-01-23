//@check-pass

use duchess::prelude::*;

duchess::java_package! {
    package java_passing_null;

    public class JavaPassingNull {
        native java.lang.String identity(java.lang.String);
    }
}

#[duchess::java_function(java_passing_null.JavaPassingNull::identity)]
fn the_identity_fn(
    _this: &java_passing_null::JavaPassingNull,
    name: Option<&java::lang::String>,
) -> duchess::Result<Option<Java<java::lang::String>>> {
    // this should be easier :)
    Ok(match name {
        Some(n) => Some(n.execute()?),
        None => None,
    })
}
