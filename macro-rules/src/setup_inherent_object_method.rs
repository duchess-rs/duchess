#[macro_export]
macro_rules! setup_inherent_object_method {
    (
        struct_name: [$S:ident],

        java_class_generics: [$($G:ident,)*],

        // Snake case version of java method name
        rust_method_name: [$M:ident],

        // Camel case version of java method name
        rust_method_generics: [$($MG:ident,)*],
        input_names: [$($I:tt,)*],
        input_ty_tts: [$($I_ty:tt,)*],
        input_ty_ops: [$($I_op:path,)*],
        output_ty_tt: [$O_ty:tt],
        sig_where_clauses: [$($SIG:tt)*],
        jni_method: [$jni_method:expr],
        jni_descriptor: [$jni_descriptor:expr],
    ) => {
        pub fn $M<$($MG,)*>(
            this: impl duchess::prelude::IntoJava<$S<$($G,)*>>,
            $($I: duchess::semver_unstable::argument_impl_trait!($I_ty),)*
        ) -> duchess::semver_unstable::output_trait!($O_ty)
        where
            $($SIG)*
        {
            pub struct $M<
                $($G,)*
                $($MG,)*
                this,
                $($I,)*
            > {
                this: this,
                $($I: $I,)*
                phantom: ::core::marker::PhantomData<($($G,)* $($MG,)*)>,
            }

            impl<$($G,)* $($MG,)* this, $($I,)*> ::core::clone::Clone
            for $M<$($G,)* $($MG,)* this, $($I,)*>
            where
                this: duchess::semver_unstable::JvmRefOp<$S<$($G,)*>>,
                $($I: $I_op,)*
                $($G: duchess::JavaObject,)*
                $($SIG)*
            {
                fn clone(&self) -> Self {
                    $M {
                        this: Clone::clone(&self.this),
                        $($I: Clone::clone(&self.$I),)*
                        phantom: self.phantom,
                    }
                }
            }

            impl<$($G,)* $($MG,)* this, $($I,)*> duchess::prelude::JvmOp
            for $M<$($G,)* $($MG,)* this, $($I,)*>
            where
                this: duchess::semver_unstable::JvmRefOp<$S<$($G,)*>>,
                $($I: $I_op,)*
                $($G: duchess::JavaObject,)*
                $($SIG)*
            {
                type Output<'jvm> = duchess::semver_unstable::output_type!('jvm, $O_ty);

                fn do_jni<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::LocalResult<'jvm, Self::Output<'jvm>> {
                    use duchess::semver_unstable::once_cell::sync::OnceCell;

                    let this = self.this.into_as_jref(jvm)?;
                    let this: &$S<$($G,)*> = duchess::prelude::AsJRef::as_jref(&this)?;
                    let this = duchess::semver_unstable::JavaObjectExt::as_raw(this);

                    $(
                        duchess::semver_unstable::prepare_input!(let $I = (self.$I: $I_ty) in jvm);
                    )*

                    // Cache the method id for this method -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static METHOD: OnceCell<duchess::semver_unstable::MethodPtr> = OnceCell::new();
                    let method = METHOD.get_or_try_init(|| {
                        let class = <$S<$($G,)*> as duchess::JavaObject>::class(jvm)?;
                        duchess::semver_unstable::find_method(jvm, &class, $jni_method, $jni_descriptor, false)
                    })?;

                    unsafe {
                        jvm.env().invoke(
                            duchess::semver_unstable::jni_call_fn!($O_ty),
                            |env, f| f(
                                env,
                                this.as_ptr(),
                                method.as_ptr(),
                                [
                                    $(duchess::semver_unstable::IntoJniValue::into_jni_value($I),)*
                                ].as_ptr(),
                            ),
                        )
                    }
                }
            }

            duchess::semver_unstable::macro_if! {
                if is_ref_ty($O_ty) {
                    impl<$($G,)* $($MG,)* this, $($I,)*> ::core::ops::Deref
                    for $M<$($G,)* $($MG,)* this, $($I,)*>
                    where
                        $($G: duchess::JavaObject,)*
                        $($SIG)*
                    {
                        type Target = duchess::semver_unstable::view_of_op!($O_ty);

                        fn deref(&self) -> &Self::Target {
                            <Self::Target as duchess::semver_unstable::FromRef<_>>::from_ref(self)
                        }
                    }
                }
            }

            $M {
                this: this.into_op(),
                $($I: $I.into_op(),)*
                phantom: ::core::default::Default::default(),
            }
        }
    };
}
