use duchess::{java, prelude::*, Global, Local};
use std::collections::HashMap;
use thiserror::Error;

duchess::java_package! {
    package auth;

    class Authenticated { * }
    class AuthorizeRequest { * }
    class HttpAuth { * }
    class HttpRequest { * }

    class AuthenticationException { * }
    class AuthenticationExceptionUnauthenticated { * }
    class AuthenticationExceptionInvalidSecurityToken { * }
    class AuthenticationExceptionInvalidSignature { * }
    class AuthorizationException { * }
    class AuthorizationExceptionDenied { * }
}

pub struct HttpAuth(Global<auth::HttpAuth>);

#[derive(Debug, duchess::ToJava)]
#[java(auth.HttpRequest)]
pub struct HttpRequest {
    pub verb: String,
    pub path: String,
    pub hashed_payload: Vec<u8>,
    pub parameters: HashMap<String, Vec<String>>,
    pub headers: HashMap<String, Vec<String>>,
}

#[derive(duchess::ToRust, duchess::ToJava)]
#[java(auth.Authenticated)]
pub struct Authenticated {
    pub account_id: String,
    pub user: String,
    this: Global<auth::Authenticated>,
}

#[derive(Debug, Error, duchess::ToRust)]
#[java(java.lang.Throwable)]
pub enum AuthenticateError {
    #[error("Unathenticated({user_message})")]
    #[java(auth.AuthenticationExceptionUnauthenticated)]
    Unathenticated { user_message: String },

    #[error("InvalidSecurityToken")]
    #[java(auth.AuthenticationExceptionInvalidSecurityToken)]
    InvalidSecurityToken,

    #[error("InvalidSignature")]
    #[java(auth.AuthenticationExceptionInvalidSignature)]
    InvalidSignature,

    #[error("Generic({get_message})")]
    #[java(auth.AuthenticationException)]
    Generic { get_message: String },

    #[error("InternalError({get_message})")]
    #[java(java.lang.Throwable)]
    InternalError { get_message: String },
}

#[derive(Debug, duchess::ToJava)]
#[java(auth.AuthorizeRequest)]
pub struct AuthorizeRequest {
    pub resource: String,
    pub action: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Error, duchess::ToRust)]
#[java(java.lang.Throwable)]
pub enum AuthorizeError {
    #[error("Denied({user_message})")]
    #[java(auth.AuthorizationExceptionDenied)]
    Denied { user_message: String },

    #[error("Generic({get_message})")]
    #[java(auth.AuthorizationException)]
    Generic { get_message: String },

    #[error("InternalError({get_message})")]
    #[java(java.lang.Throwable)]
    InternalError { get_message: String },
}

impl HttpAuth {
    pub fn new() -> duchess::GlobalResult<Self> {
        let auth = auth::HttpAuth::new().global().execute()?;
        Ok(Self(auth))
    }

    pub fn authenticate(&self, request: &HttpRequest) -> Result<Authenticated, AuthenticateError> {
        self.0
            .authenticate(request)
            .assert_not_null()
            .catch::<duchess::java::lang::Throwable>()
            .to_rust()
            .execute()
            .unwrap()
    }

    pub fn authorize(
        &self,
        authn: &Authenticated,
        authz: &AuthorizeRequest,
    ) -> Result<(), AuthorizeError> {
        self.0
            .authorize(authn, authz)
            .catch::<duchess::java::lang::Throwable>()
            .to_rust()
            .execute()
            .unwrap()
    }
}

fn main() -> duchess::GlobalResult<()> {
    let auth = HttpAuth::new()?;

    let request = HttpRequest {
        verb: "POST".into(),
        path: "/".into(),
        hashed_payload: vec![1, 2, 3],
        parameters: HashMap::new(),
        headers: [("Authentication".into(), vec!["Some signature".into()])].into(),
    };

    let authenticated = match auth.authenticate(&request) {
        Ok(a) => a,
        Err(e) => {
            println!("couldn't authenticate: {:#?}", e);
            return Ok(());
        }
    };

    println!(
        "User `{}` in `{}` authenticated",
        authenticated.user, authenticated.account_id
    );

    let request = AuthorizeRequest {
        resource: "my-resource".into(),
        action: "delete".into(),
        context: HashMap::new(),
    };

    if let Err(e) = auth.authorize(&authenticated, &request) {
        println!("User `{}` access denied: {:?}", authenticated.user, e);
        return Ok(());
    }
    println!("User allowed to delete my-resource");

    Ok(())
}
