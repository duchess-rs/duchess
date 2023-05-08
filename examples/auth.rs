use duchess::java::lang::ThrowableExt;
use duchess::java::util::{ArrayList, ArrayListExt, HashMap as JavaHashMap, MapExt};
use duchess::{prelude::*, Global, Jvm, Local, ToRust};
use std::collections::HashMap;
use thiserror::Error;

// XX: should we automatically attach allow(dead_code)?
#[allow(dead_code)]
mod java_auth {
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

    // XX: can be removed when we automatically look through extends/implements
    use duchess::java;
    unsafe impl duchess::plumbing::Upcast<java::lang::Throwable>
        for AuthenticationExceptionUnauthenticated
    {
    }
    unsafe impl duchess::plumbing::Upcast<java::lang::Throwable>
        for AuthenticationExceptionInvalidSecurityToken
    {
    }
    unsafe impl duchess::plumbing::Upcast<java::lang::Throwable>
        for AuthenticationExceptionInvalidSignature
    {
    }
    unsafe impl duchess::plumbing::Upcast<java::lang::Throwable> for AuthorizationExceptionDenied {}

    pub use auth::*;
}

use java_auth::{
    AuthenticatedExt, AuthenticationExceptionUnauthenticatedExt, AuthorizationExceptionDeniedExt,
    HttpAuthExt,
};

pub struct HttpAuth(Global<java_auth::HttpAuth>);

#[derive(Debug)]
pub struct HttpRequest {
    pub verb: String,
    pub path: String,
    pub body_hash: Vec<u8>,
    pub params: HashMap<String, Vec<String>>,
    pub headers: HashMap<String, Vec<String>>,
}

pub struct Authenticated {
    pub account_id: String,
    pub user: String,
    this: Global<java_auth::Authenticated>,
}

#[derive(Debug, Error)]
pub enum AuthenticateError {
    #[error("Unathenticated({0})")]
    Unathenticated(String),
    #[error("InvalidSecurityToken")]
    InvalidSecurityToken,
    #[error("InvalidSignature")]
    InvalidSignature,
    #[error("InternalError({0})")]
    InternalError(String),
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
        let auth = Jvm::with(|jvm| {
            let auth = java_auth::HttpAuth::new().execute_with(jvm)?;
            Ok(jvm.global(&*auth))
        })?;
        Ok(Self(auth))
    }

    pub fn authenticate(&self, request: &HttpRequest) -> Result<Authenticated, AuthenticateError> {
        Jvm::with(|jvm| {
            self.0
                .authenticate(request)
                .assert_not_null()
                .catch::<duchess::java::lang::Throwable>()
                .to_rust()
                .execute_with(jvm)
        })
        .unwrap()
    }

    pub fn authorize(
        &self,
        authn: &Authenticated,
        authz: &AuthorizeRequest,
    ) -> Result<(), AuthorizeError> {
        Jvm::with(
            |jvm| match self.0.authorize(authn, authz).execute_with(jvm) {
                Ok(()) => Ok(Ok(())),
                Err(duchess::Error::Thrown(exception)) => Ok(Err(
                    if let Ok(x) = exception
                        .try_downcast::<java_auth::AuthorizationExceptionDenied>()
                        .execute_with(jvm)?
                    {
                        let message = x
                            .user_message()
                            .assert_not_null()
                            .to_rust()
                            .execute_with(jvm)?;
                        AuthorizeError::Denied(message)
                    } else {
                        let message = exception
                            .get_message()
                            .assert_not_null()
                            .to_rust()
                            .execute_with(jvm)?;
                        AuthorizeError::InternalError(message)
                    },
                )),

                Err(e) => Err(e),
            },
        )
        .unwrap()
    }
}

// XX: Could we build a #[derive(IntoJava)] macro to remove a lot this boiler plate? Or perhaps for data-only classes
// the javap macro could build these?
impl JvmOp for &HttpRequest {
    type Output<'jvm> = Local<'jvm, java_auth::HttpRequest>;

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, Self::Output<'jvm>> {
        // XX: we should provide utils for constructing java maps and lists
        let java_params = JavaHashMap::new().execute_with(jvm)?;
        for (param, values) in &self.params {
            let java_values = ArrayList::new().execute_with(jvm)?;
            for value in values {
                // XX: can we remove explicit .as_str()?
                java_values.add(value.as_str()).execute_with(jvm)?;
            }
            java_params
                .put(param.as_str(), &java_values)
                .execute_with(jvm)?;
        }

        let java_headers = JavaHashMap::new().execute_with(jvm)?;
        for (header, values) in &self.headers {
            let java_values = ArrayList::new().execute_with(jvm)?;
            for value in values {
                java_values.add(value.as_str()).execute_with(jvm)?;
            }
            java_headers
                .put(header.as_str(), &java_values)
                .execute_with(jvm)?;
        }

        // XX: should we allow &[u8] to work automatically for byte[]?
        let body_hash_signed = self.body_hash.iter().map(|&b| b as i8).collect::<Vec<_>>();
        java_auth::HttpRequest::new(
            self.verb.as_str(),
            self.path.as_str(),
            body_hash_signed.as_slice(),
            &java_params,
            &java_headers,
        )
        .execute_with(jvm)
    }
}

impl ToRust<Authenticated> for java_auth::Authenticated {
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, Authenticated> {
        let account_id = self
            .account_id()
            .assert_not_null()
            .to_rust()
            .execute_with(jvm)?;
        let user = self.user().assert_not_null().to_rust().execute_with(jvm)?;
        let this = self.global().execute_with(jvm)?;
        Ok(Authenticated {
            account_id,
            user,
            this,
        })
    }
}

impl ToRust<AuthenticateError> for duchess::java::lang::Throwable {
    fn to_rust<'jvm>(&self, jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, AuthenticateError> {
        // XX: why can't we infer the <Throwable, ?
        if let Ok(x) = self
            .try_downcast::<java_auth::AuthenticationExceptionUnauthenticated>()
            .execute_with(jvm)?
        {
            let message = x
                .user_message()
                .assert_not_null()
                .to_rust()
                .execute_with(jvm)?;
            Ok(AuthenticateError::InternalError(message))
        // XX: should we add a .is_instance() alias for try_downcast().is_ok()?
        } else if self
            .try_downcast::<java_auth::AuthenticationExceptionInvalidSecurityToken>()
            .execute_with(jvm)?
            .is_ok()
        {
            Ok(AuthenticateError::InvalidSecurityToken)
        } else if self
            .try_downcast::<java_auth::AuthenticationExceptionInvalidSignature>()
            .execute_with(jvm)?
            .is_ok()
        {
            Ok(AuthenticateError::InvalidSignature)
        } else {
            let message = self
                .get_message()
                .assert_not_null()
                .to_rust()
                .execute_with(jvm)?;
            Ok(AuthenticateError::InternalError(message))
        }
    }
}

impl<'a> JvmOp for &'a Authenticated {
    type Output<'jvm> = &'a Global<java_auth::Authenticated>;

    fn execute_with<'jvm>(self, _jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, Self::Output<'jvm>> {
        Ok(&self.this)
    }
}

impl JvmOp for &AuthorizeRequest {
    type Output<'jvm> = Local<'jvm, java_auth::AuthorizeRequest>;

    fn execute_with<'jvm>(self, jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, Self::Output<'jvm>> {
        let java_context = JavaHashMap::new().execute_with(jvm)?;
        for (key, value) in &self.context {
            java_context
                .put(key.as_str(), value.as_str())
                .execute_with(jvm)?;
        }

        java_auth::AuthorizeRequest::new(
            self.resource.as_str(),
            self.action.as_str(),
            &java_context,
        )
        .execute_with(jvm)
    }
}

fn main() -> duchess::GlobalResult<()> {
    let auth = HttpAuth::new()?;

    let request = HttpRequest {
        verb: "POST".into(),
        path: "/".into(),
        body_hash: vec![1, 2, 3],
        params: HashMap::new(),
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
