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

fn main() -> jni::errors::Result<()> {
    Jvm::with(|jvm| {
        let params = HashMap::new().execute(jvm)?;
        let values = ArrayList::new().execute(jvm)?;
        values.add("first-value").execute(jvm)?;
        values.add("second-value").execute(jvm)?;
        params.put("first-param", &values).execute(jvm)?;

        let http_request =
            HttpRequest::new("POST", "/", [1i8, 2, 3].as_slice(), &params).execute(jvm)?;

        let as_str = http_request.to_string().assert_not_null().into_rust(jvm)?;
        println!("{}", as_str);

        Ok(())
    })
}
