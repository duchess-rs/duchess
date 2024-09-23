#[macro_export]
macro_rules! setup_inherent_object_method {
    (
        struct_name: [$S:ident],

        java_class_generics: [$($G:ident,)*],

        // Snake case version of java method name
        rust_method_name: [$rust_method_name:ident],

        // Camel case version of java method name
        rust_method_struct_name: [$rust_method_struct_name:ident],
        rust_method_generics: [$($MG:ident,)*],
        input_names: [$($I:ident,)*],
        input_traits: [$($input_trait:path,)*],
        jvm_op_traits: [$($jvm_op_trait:path,)*],
        output_ty: [$output_ty:ty],
        output_trait: [$output_trait:path],
        java_ref_output_ty: [$($java_ref_output_ty:tt)*],
        sig_where_clauses: [$($sig_where_clause:tt)*],
        prepare_inputs: [$($prepare_inputs:tt)*],
        jni_call_fn: [$jni_call_fn:ident],
        jni_method: [$jni_method:expr],
        jni_descriptor: [$jni_descriptor:expr],
        idents: [$self:ident, $jvm:ident],
    ) => {
        pub fn $rust_method_name<$($MG,)*>(
            this: impl duchess::prelude::IntoJava<$S<$($G,)*>>,
            $($I: impl $input_trait),*
        ) -> impl $output_trait
        where
            $($sig_where_clause)*
        {
            pub struct $rust_method_struct_name<
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
            for $rust_method_struct_name<$($G,)* $($MG,)* this, $($I,)*>
            where
                this: duchess::plumbing::JvmRefOp<$S<$($G,)*>>,
                $($I: $jvm_op_trait,)*
                $($G: duchess::JavaObject,)*
                $($sig_where_clause)*
            {
                fn clone(&self) -> Self {
                    $rust_method_struct_name {
                        this: Clone::clone(&self.this),
                        $($I: Clone::clone(&self.$I),)*
                        phantom: self.phantom,
                    }
                }
            }

            impl<$($G,)* $($MG,)* this, $($I,)*> duchess::prelude::JvmOp
            for $rust_method_struct_name<$($G,)* $($MG,)* this, $($I,)*>
            where
                this: duchess::plumbing::JvmRefOp<$S<$($G,)*>>,
                $($I: $jvm_op_trait,)*
                $($G: duchess::JavaObject,)*
                $($sig_where_clause)*
            {
                type Output<'jvm> = $output_ty;

                fn do_jni<'jvm>(
                    $self,
                    $jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::LocalResult<'jvm, Self::Output<'jvm>> {
                    use duchess::plumbing::once_cell::sync::OnceCell;

                    let this = $self.this.into_as_jref($jvm)?;
                    let this: &$S<$($G,)*> = duchess::prelude::AsJRef::as_jref(&this)?;
                    let this = duchess::plumbing::JavaObjectExt::as_raw(this);

                    $($prepare_inputs)*

                    // Cache the method id for this method -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static METHOD: OnceCell<duchess::plumbing::MethodPtr> = OnceCell::new();
                    let method = METHOD.get_or_try_init(|| {
                        let class = <$S<$($G,)*> as duchess::JavaObject>::class($jvm)?;
                        duchess::plumbing::find_method($jvm, &class, $jni_method, $jni_descriptor, false)
                    })?;

                    unsafe {
                        $jvm.env().invoke(|env| env.$jni_call_fn, |env, f| f(
                            env,
                            this.as_ptr(),
                            method.as_ptr(),
                            [
                                $(duchess::plumbing::IntoJniValue::into_jni_value($I),)*
                            ].as_ptr(),
                        ))
                    }
                }
            }

            duchess::plumbing::macro_if! {
                if [$($java_ref_output_ty)*] {
                    impl<$($G,)* $($MG,)* this, $($I,)*> ::core::ops::Deref
                    for $rust_method_struct_name<$($G,)* $($MG,)* this, $($I,)*>
                    where
                        $($G: duchess::JavaObject,)*
                        $($sig_where_clause)*
                    {
                        type Target = <$($java_ref_output_ty)* as duchess::plumbing::JavaView>::OfOp<Self>;

                        fn deref(&self) -> &Self::Target {
                            <Self::Target as duchess::plumbing::FromRef<_>>::from_ref(self)
                        }
                    }
                }
            }

            $rust_method_struct_name {
                this: this.into_op(),
                $($I: $I.into_op(),)*
                phantom: ::core::default::Default::default(),
            }
        }
    };
}
