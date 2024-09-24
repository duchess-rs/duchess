#[macro_export]
macro_rules! setup_static_field_getter {
    (
        struct_name: [$S:ident],
        java_class_generics: [$($G:ident,)*],
        rust_field_name: [$rust_field_name:ident],
        rust_field_struct_name: [$rust_field_struct_name:ident],
        output_ty: [$output_ty:ty],
        output_trait: [$output_trait:path],
        sig_where_clauses: [$($sig_where_clause:tt)*],
        jni_field: [$jni_field:expr],
        jni_descriptor: [$jni_descriptor:expr],
        jni_field_fn: [$jni_field_fn:ident],
    ) => {
        pub fn $rust_field_name() -> impl $output_trait
        where
            $($sig_where_clause)*
        {
            pub struct $rust_field_struct_name<
                $($G,)*
            > {
                phantom: ::core::marker::PhantomData<(
                    $($G,)*
                )>,
            }

            impl<$($G,)*> duchess::prelude::JvmOp
            for $rust_field_struct_name<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
                $($sig_where_clause)*
            {
                type Output<'jvm> = $output_ty;

                fn do_jni<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::LocalResult<'jvm, Self::Output<'jvm>> {
                    use duchess::plumbing::once_cell::sync::OnceCell;

                    // Cache the field id for this field -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static FIELD: OnceCell<duchess::plumbing::FieldPtr> = OnceCell::new();
                    let field = FIELD.get_or_try_init(|| {
                        let class = <$S<$($G,)*> as duchess::JavaObject>::class(jvm)?;
                        duchess::plumbing::find_field(jvm, &class, $jni_field, $jni_descriptor, true)
                    })?;

                    let class = <$S<$($G,)*> as duchess::JavaObject>::class(jvm)?;
                    unsafe {
                        jvm.env().invoke(|env| env.$jni_field_fn, |env, f| f(
                            env,
                            duchess::plumbing::JavaObjectExt::as_raw(&*class).as_ptr(),
                            field.as_ptr(),
                        ))
                    }
                }
            }


            impl<$($G,)*> ::core::clone::Clone for $rust_field_struct_name<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
                $($sig_where_clause)*
            {
                fn clone(&self) -> Self {
                    $rust_field_struct_name {
                        phantom: self.phantom,
                    }
                }
            }

            $rust_field_struct_name {
                phantom: ::core::default::Default::default(),
            }
        }
    };
}
