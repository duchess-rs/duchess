#[macro_export]
macro_rules! setup_static_field_getter {
    (
        struct_name: [$S:ident],
        java_class_generics: [$($G:ident,)*],
        rust_field_name: [$F:ident],
        field_ty: [$F_ty:tt],
        sig_where_clauses: [$($SIG:tt)*],
        jni_field: [$jni_field:expr],
        jni_descriptor: [$jni_descriptor:expr],
    ) => {
        pub fn $F() -> duchess::semver_unstable::field_output_trait!($F_ty)
        where
            $($SIG)*
        {
            #[allow(non_camel_case_types)]
            pub struct $F<
                $($G,)*
            > {
                phantom: ::core::marker::PhantomData<(
                    $($G,)*
                )>,
            }

            impl<$($G,)*> duchess::prelude::JvmOp
            for $F<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
                $($SIG)*
            {
                type Output<'jvm> = duchess::semver_unstable::output_type!('jvm, $F_ty);

                fn do_jni<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::LocalResult<'jvm, Self::Output<'jvm>> {
                    use duchess::semver_unstable::once_cell::sync::OnceCell;

                    // Cache the field id for this field -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static FIELD: OnceCell<duchess::semver_unstable::FieldPtr> = OnceCell::new();
                    let field = FIELD.get_or_try_init(|| {
                        let class = <$S<$($G,)*> as duchess::JavaObject>::class(jvm)?;
                        duchess::semver_unstable::find_field(jvm, &class, $jni_field, $jni_descriptor, true)
                    })?;

                    let class = <$S<$($G,)*> as duchess::JavaObject>::class(jvm)?;
                    unsafe {
                        jvm.env().invoke(
                            duchess::semver_unstable::jni_static_field_get_fn!($F_ty),
                            |env, f| f(
                                env,
                                duchess::semver_unstable::JavaObjectExt::as_raw(&*class).as_ptr(),
                                field.as_ptr(),
                            ),
                        )
                    }
                }
            }


            impl<$($G,)*> ::core::clone::Clone for $F<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
                $($SIG)*
            {
                fn clone(&self) -> Self {
                    $F {
                        phantom: self.phantom,
                    }
                }
            }

            $F {
                phantom: ::core::default::Default::default(),
            }
        }
    };
}
