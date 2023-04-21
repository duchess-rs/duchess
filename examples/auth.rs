use duchess::{
    java::util::{ArrayList, HashMap, ListExt, MapExt},
    prelude::*,
    Jvm,
};

duchess::java_package! {
    package me.ferris;

    class AuthenticateResult { * }
    class HttpAuth { * }
    class HttpRequest { * }
}

use me::ferris::*;

fn perform_auth_request() -> duchess::GlobalResult<String> {
    Jvm::with(|jvm| {
        let params = HashMap::new().execute(jvm)?;
        let values = ArrayList::new().execute(jvm)?;
        values.add("first-value").execute(jvm)?;
        values.add("second-value").execute(jvm)?;
        params.put("first-param", &values).execute(jvm)?;

        let http_request =
            HttpRequest::new("POST", "/", [1i8, 2, 3].as_slice(), &params).execute(jvm)?;

        Ok(http_request.to_string().assert_not_null().into_rust(jvm)?)
    })
}

fn main() -> duchess::GlobalResult<()> {
    let s = perform_auth_request()?;
    println!("{s}");
    Ok(())
}

#[test]
fn invoke() {
    expect_test::expect![[r#"
        Ok(
            "HttpRequest[verb=POST, path=/, hashedPayload=[B@2fc98066, parameters={first-param=[first-value, second-value]}]",
        )
    "#]].assert_debug_eq(&perform_auth_request());
}
