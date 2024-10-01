#[macro_export]
macro_rules! setup_constructor {
    (
        struct_name: [$S:ident],
        java_class_generics: [$($G:ident,)*],
        input_names: [$($I:ident,)*],
        input_ty_tts: [$($I_ty:tt,)*],
        input_ty_ops: [$($I_op:path,)*],
        descriptor: [$descriptor:expr],
        jni_descriptor: [$jni_descriptor:expr],
    ) => {
        pub fn new(
            $($I : duchess::plumbing::argument_impl_trait!($I_ty),)*
        ) -> impl duchess::prelude::JavaConstructor<$S<$($G,)*>> {
            struct Impl<
                $($G,)*
                $($I,)*
            > {
                $($I : $I,)*
                phantom: ::core::marker::PhantomData<($($G,)*)>,
            }

            impl<$($G,)* $($I,)*> ::core::clone::Clone for Impl<$($G,)* $($I,)*>
            where
                $($G: duchess::JavaObject,)*
                $($I: $I_op,)*
            {
                fn clone(&self) -> Self {
                    Impl {
                        $($I : ::core::clone::Clone::clone(&self.$I),)*
                        phantom: ::core::marker::PhantomData,
                    }
                }
            }

            impl<$($G,)* $($I,)*> duchess::prelude::JvmOp for Impl<$($G,)* $($I,)*>
            where
                $($G: duchess::JavaObject,)*
                $($I: $I_op,)*
            {
                type Output<'jvm> = duchess::Local<'jvm, $S<$($G,)*>>;

                fn do_jni<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::LocalResult<'jvm, Self::Output<'jvm>> {
                    use duchess::plumbing::once_cell::sync::OnceCell;

                    $(
                        duchess::plumbing::prepare_input!(let $I = (self.$I: $I_ty) in jvm);
                    )*

                    let class = <$S<$($G,)*> as duchess::JavaObject>::class(jvm)?;

                    // Cache the method id for the constructor -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static CONSTRUCTOR: OnceCell<duchess::plumbing::MethodPtr> = OnceCell::new();
                    let constructor = CONSTRUCTOR.get_or_try_init(|| {
                        duchess::plumbing::find_constructor(jvm, &class, $jni_descriptor)
                    })?;

                    let env = jvm.env();
                    let obj: ::core::option::Option<duchess::Local<$S<$($G,)*>>> = unsafe {
                        env.invoke(|env| env.NewObjectA, |env, f| f(
                            env,
                            duchess::plumbing::JavaObjectExt::as_raw(&*class).as_ptr(),
                            constructor.as_ptr(),
                            [
                                $(duchess::plumbing::IntoJniValue::into_jni_value($I),)*
                            ].as_ptr(),
                        ))
                    }?;
                    obj.ok_or_else(|| {
                        // NewObjectA should only return a null pointer when an exception occurred in the
                        // constructor, so reaching here is a strange JVM state
                        duchess::Error::JvmInternal(format!(
                            "failed to create new `{}` via constructor `{}`",
                            stringify!($S), $descriptor,
                        ))
                    })
                }
            }


            impl<$($G,)* $($I,)*> ::core::ops::Deref for Impl<$($G,)* $($I,)*>
            where
                $($G: duchess::JavaObject,)*
                $($I: $I_op,)*
            {
                type Target = <$S<$($G,)*> as duchess::plumbing::JavaView>::OfOp<Self>;

                fn deref(&self) -> &Self::Target {
                    <Self::Target as duchess::plumbing::FromRef<_>>::from_ref(self)
                }
            }

            Impl {
                $($I: $I.into_op(),)*
                phantom: ::core::default::Default::default()
            }
        }
    }
}
