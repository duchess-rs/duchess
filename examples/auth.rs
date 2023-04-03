use duchess::{
    plumbing::{IntoJavaArray, IntoJavaString, JavaObjectExt, JavaString},
    IntoRust, JavaObject, Jvm, JvmOp, Local,
};
use jni::{
    objects::{AutoLocal, GlobalRef, JMethodID, JValueGen},
    signature::ReturnType,
    sys::jvalue,
};
use once_cell::sync::OnceCell;

pub struct HttpRequest(());

unsafe impl JavaObject for HttpRequest {}

impl HttpRequest {
    pub fn new(
        verb: impl IntoJavaString,
        path: impl IntoJavaString,
        hashed_payload: impl IntoJavaArray<i8>,
    ) -> impl for<'jvm> duchess::JvmOp<Input<'jvm> = (), Output<'jvm> = Local<'jvm, HttpRequest>>
    {
        #[derive(Clone)]
        struct Impl<Verb, Path, HashedPayload> {
            verb: Verb,
            path: Path,
            hashed_payload: HashedPayload,
        }

        impl<Verb, Path, HashedPayload> JvmOp for Impl<Verb, Path, HashedPayload>
        where
            Verb: IntoJavaString,
            Path: IntoJavaString,
            HashedPayload: IntoJavaArray<i8>,
        {
            type Input<'jvm> = ();
            type Output<'jvm> = Local<'jvm, HttpRequest>;

            fn execute_with<'jvm>(
                self,
                jvm: &mut duchess::Jvm<'jvm>,
                (): (),
            ) -> duchess::Result<Self::Output<'jvm>> {
                let verb = self.verb.into_java(jvm)?;
                let path = self.path.into_java(jvm)?;
                let hashed_payload = self.hashed_payload.into_java(jvm)?;

                let class = HttpRequest::cached_class(jvm)?;

                let env = jvm.to_env();
                static CONSTRUCTOR: OnceCell<JMethodID> = OnceCell::new();
                let constructor = CONSTRUCTOR.get_or_try_init(|| {
                    env.get_method_id(class, "<init>", "(Ljava/lang/String;Ljava/lang/String;[B)V")
                })?;

                let object = unsafe {
                    env.new_object_unchecked(
                        class,
                        *constructor,
                        &[
                            jvalue {
                                l: verb.as_ref().as_jobject().as_raw(),
                            },
                            jvalue {
                                l: path.as_ref().as_jobject().as_raw(),
                            },
                            jvalue {
                                l: hashed_payload.as_ref().as_jobject().as_raw(),
                            },
                        ],
                    )?
                };

                Ok(unsafe { Local::from_jni(AutoLocal::new(object, &env)) })
            }
        }

        Impl {
            verb,
            path,
            hashed_payload,
        }
    }

    fn cached_class(jvm: &mut Jvm<'_>) -> duchess::Result<&'static GlobalRef> {
        let env = jvm.to_env();

        static CLASS: OnceCell<GlobalRef> = OnceCell::new();
        CLASS.get_or_try_init(|| {
            let class = env.find_class("me/ferris/HttpRequest")?;
            env.new_global_ref(class)
        })
    }
}

pub trait HttpRequestExt: JvmOp + Sized {
    fn to_string(self) -> HttpRequestToString<Self> {
        HttpRequestToString { this: self }
    }
}

impl<T: JvmOp> HttpRequestExt for T where for<'jvm> T::Output<'jvm>: AsRef<HttpRequest> {}

#[derive(Clone)]
pub struct HttpRequestToString<T> {
    this: T,
}

impl<T: JvmOp> JvmOp for HttpRequestToString<T>
where
    for<'jvm> T::Output<'jvm>: AsRef<HttpRequest>,
{
    type Input<'jvm> = T::Input<'jvm>;
    type Output<'jvm> = Local<'jvm, JavaString>;

    fn execute_with<'jvm>(
        self,
        jvm: &mut Jvm<'jvm>,
        input: Self::Input<'jvm>,
    ) -> duchess::Result<Self::Output<'jvm>> {
        let this = self.this.execute_with(jvm, input)?;

        let class = HttpRequest::cached_class(jvm)?;

        let env = jvm.to_env();
        static METHOD: OnceCell<JMethodID> = OnceCell::new();
        let method = METHOD
            .get_or_try_init(|| env.get_method_id(class, "toString", "()Ljava/lang/String;"))?;

        let result = unsafe {
            env.call_method_unchecked(this.as_ref().as_jobject(), *method, ReturnType::Object, &[])?
        };
        let JValueGen::Object(result) = result else {
            panic!("expected object for toString() result");
        };

        Ok(unsafe { Local::from_jni(AutoLocal::new(result, env)) })
    }
}

fn main() -> jni::errors::Result<()> {
    Jvm::with(|jvm| {
        let http_request = HttpRequest::new("POST", "/", [1i8, 2, 3].as_slice()).execute(jvm)?;

        let as_str = http_request.to_string().into_rust().execute(jvm)?;
        println!("{}", as_str);

        Ok(())
    })
}
