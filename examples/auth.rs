use duchess::java::lang::{Throwable, ThrowableExt};
use duchess::java::util::{ArrayList, ArrayListExt, HashMap as JavaHashMap, MapExt};
use std::collections::HashMap;

use duchess::{prelude::*, Global, Jvm, Local};

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
    state: Global<java_auth::Authenticated>,
}

#[derive(Debug)]
pub enum AuthenticateError {
    Unathenticated(String),
    InvalidSecurityToken,
    InvalidSignature,
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
            let auth = java_auth::HttpAuth::new().execute(jvm)?;
            Ok(jvm.global(&*auth))
        })?;
        Ok(Self(auth))
    }

    pub fn authenticate(&self, request: &HttpRequest) -> Result<Authenticated, AuthenticateError> {
        Jvm::with(|jvm| {
            let request = request.into_java(jvm)?;
            match self.0.authenticate(&request).assert_not_null().execute(jvm) {
                Ok(auth) => Ok(Ok(Authenticated::from_java(jvm, auth)?)),
                Err(duchess::Error::Thrown(exception)) => {
                    // XX: is this kind of type switching better handled by a macro?
                    Ok(Err(
                        // XX: why can't we infer the <Throwable, ? 
                        if let Ok(x) = exception.try_downcast::<Throwable, java_auth::AuthenticationExceptionUnauthenticated>().execute(jvm)? {
                            let message = x.user_message().assert_not_null().into_rust(jvm)?;
                            AuthenticateError::Unathenticated(message)
                        // XX: should we add a .is_instance() alias for try_downcast().is_ok()?
                        } else if exception.try_downcast::<Throwable, java_auth::AuthenticationExceptionInvalidSecurityToken>().execute(jvm)?.is_ok() {
                            AuthenticateError::InvalidSecurityToken
                        } else if exception.try_downcast::<Throwable, java_auth::AuthenticationExceptionInvalidSignature>().execute(jvm)?.is_ok() {
                            AuthenticateError::InvalidSignature
                        } else {
                            let message = exception.get_message().assert_not_null().into_rust(jvm)?;
                            AuthenticateError::InternalError(message)
                        }
                    ))
                }
                // XX: do we want to hide null derefs and other JNI problems? or should they get converted to something 
                // visible to the user?
                Err(e) => Err(e),
            }
        // XX: we should implement a helpful Display/Debug for Error that isn't just "Thrown"
        }).unwrap()
    }

    pub fn authorize(
        &self,
        authn: &Authenticated,
        authz: &AuthorizeRequest,
    ) -> Result<(), AuthorizeError> {
        Jvm::with(|jvm| {
            let authz = authz.into_java(jvm)?;
            match self.0.authorize(&authn.state, &authz).execute(jvm) {
                Ok(()) => Ok(Ok(())),
                Err(duchess::Error::Thrown(exception)) => Ok(Err(
                    if let Ok(x) = exception
                        .try_downcast::<Throwable, java_auth::AuthorizationExceptionDenied>()
                        .execute(jvm)?
                    {
                        let message = x.user_message().assert_not_null().into_rust(jvm)?;
                        AuthorizeError::Denied(message)
                    } else {
                        let message = exception.get_message().assert_not_null().into_rust(jvm)?;
                        AuthorizeError::InternalError(message)
                    },
                )),

                Err(e) => Err(e),
            }
        })
        .unwrap()
    }
}

// XX: Could we build a #[derive(IntoJava)] macro to remove a lot this boiler plate? Or perhaps for data-only classes
// the javap macro could build these?
impl HttpRequest {
    fn into_java<'jvm>(
        &self,
        jvm: &mut Jvm<'jvm>,
    ) -> duchess::Result<'jvm, Local<'jvm, java_auth::HttpRequest>> {
        // XX: we should provide utils for constructing java maps and lists
        let java_params = JavaHashMap::new().execute(jvm)?;
        for (param, values) in &self.params {
            let java_values = ArrayList::new().execute(jvm)?;
            for value in values {
                // XX: can we remove explicit .as_str()?
                java_values.add(value.as_str()).execute(jvm)?;
            }
            java_params.put(param.as_str(), &java_values).execute(jvm)?;
        }

        let java_headers = JavaHashMap::new().execute(jvm)?;
        for (header, values) in &self.headers {
            let java_values = ArrayList::new().execute(jvm)?;
            for value in values {
                java_values.add(value.as_str()).execute(jvm)?;
            }
            java_headers
                .put(header.as_str(), &java_values)
                .execute(jvm)?;
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
        .execute(jvm)
    }
}

impl Authenticated {
    // XX: &java_auth:Authenticated doesn't work with AuthenticatedExt::account_id(), nor does impl AsRef<>
    // error[E0599]: the method `account_id` exists for reference `&Authenticated`, but its trait bounds were not satisfied
    //     --> examples/auth.rs:122:31
    //     |
    // 11  |         class Authenticated { * }
    //     |               -------------
    //     |               |
    //     |               doesn't satisfy `ferris::Authenticated: duchess::JvmOp`
    //     |               doesn't satisfy `ferris::Authenticated: ferris::AuthenticatedExt`
    //  ...
    // 122 |         let account_id = auth.account_id().assert_not_null().into_rust(jvm)?;
    //     |                               ^^^^^^^^^^ method cannot be called on `&Authenticated` due to unsatisfied trait bounds
    //     |
    fn from_java<'jvm>(
        jvm: &mut Jvm<'jvm>,
        auth: Local<'jvm, java_auth::Authenticated>,
    ) -> duchess::Result<'jvm, Self> {
        let account_id = auth.account_id().assert_not_null().into_rust(jvm)?;
        let user = auth.user().assert_not_null().into_rust(jvm)?;
        let state = jvm.global(&*auth);
        Ok(Self {
            account_id,
            user,
            state,
        })
    }
}

impl AuthorizeRequest {
    fn into_java<'jvm>(
        &self,
        jvm: &mut Jvm<'jvm>,
    ) -> duchess::Result<'jvm, Local<'jvm, java_auth::AuthorizeRequest>> {
        let java_context = JavaHashMap::new().execute(jvm)?;
        for (key, value) in &self.context {
            java_context
                .put(key.as_str(), value.as_str())
                .execute(jvm)?;
        }

        java_auth::AuthorizeRequest::new(
            self.resource.as_str(),
            self.action.as_str(),
            &java_context,
        )
        .execute(jvm)
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
