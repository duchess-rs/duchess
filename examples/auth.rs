use duchess::java::lang::ThrowableExt;
use duchess::java::util::{HashMap as JavaHashMap, MapExt};
use duchess::{java, prelude::*, Global, Jvm, Local, ToRust};
use std::collections::HashMap;
use thiserror::Error;

use auth::{AuthorizationExceptionDeniedExt, HttpAuthExt};

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

#[derive(Debug)]
pub struct AuthorizeRequest {
    pub resource: String,
    pub action: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug)]
pub enum AuthorizeError {
    Denied(String),
    InternalError(String),
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

    // pub fn authorize(
    //     &self,
    //     authn: &Authenticated,
    //     authz: &AuthorizeRequest,
    // ) -> Result<(), AuthorizeError> {
    //     self.0
    //         .authorize(authn, authz)
    //         .catch::<duchess::java::lang::Throwable>()
    //         .to_rust()
    //         .execute()
    //         .unwrap()
    // }
}

// impl ToRust<AuthorizeError> for duchess::java::lang::Throwable {
//     fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, AuthorizeError> {
//         if let Ok(x) = self
//             .try_downcast::<auth::AuthorizationExceptionDenied>()
//             .execute_with(jvm)?
//         {
//             let message = x
//                 .user_message()
//                 .assert_not_null()
//                 .to_rust()
//                 .execute_with(jvm)?;
//             Ok(AuthorizeError::Denied(message))
//         } else {
//             let message = self
//                 .get_message()
//                 .assert_not_null()
//                 .to_rust()
//                 .execute_with(jvm)?;
//             Ok(AuthorizeError::InternalError(message))
//         }
//     }
// }

impl JvmOp for &AuthorizeRequest {
    type Output<'jvm> = Local<'jvm, auth::AuthorizeRequest>;

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, Self::Output<'jvm>> {
        let java_context = JavaHashMap::new().execute_with(jvm)?;
        for (key, value) in &self.context {
            java_context
                .put(key.as_str(), value.as_str())
                .execute_with(jvm)?;
        }

        auth::AuthorizeRequest::new(self.resource.as_str(), self.action.as_str(), &java_context)
            .execute_with(jvm)
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

    // if let Err(e) = auth.authorize(&authenticated, &request) {
    //     println!("User `{}` access denied: {:?}", authenticated.user, e);
    //     return Ok(());
    // }
    println!("User allowed to delete my-resource");

    Ok(())
}
