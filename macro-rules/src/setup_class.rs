#[macro_export]
macro_rules! setup_class {
    (
        struct_name: [$struct_name:ident],
        java_class_generics: [$($java_class_generics:ident,)*],
        jni_class_name: [$jni_class_name:expr],
        mro_tys: [$($mro_ty:ty,)*],
        constructors: [$($constructors:tt)*],
        static_methods: [$($static_methods:tt)*],
        static_field_getters: [$($static_field_getters:tt)*],
        inherent_object_methods: [$($inherent_object_methods:tt)*],
        op_struct_methods: [$($op_struct_methods:tt)*],
        obj_struct_methods: [$($obj_struct_methods:tt)*],
        op_name: [$op_name:ident],
        obj_name: [$obj_name:ident],
    ) => {
        #[allow(non_camel_case_types)]
        pub struct $struct_name<$($java_class_generics = duchess::java::lang::Object,)*> {
            _empty: std::convert::Infallible,
            _dummy: ::core::marker::PhantomData<($($java_class_generics,)*)>
        }

        // Hide other generated items
        #[allow(unused_imports)]
        #[allow(nonstandard_style)]
        const _: () = {
            use duchess::{java, Java, Jvm, Local, LocalResult};
            use duchess::plumbing::once_cell::sync::OnceCell;
            use duchess::plumbing::mro;

            unsafe impl<$($java_class_generics,)*> duchess::JavaObject for $struct_name<$($java_class_generics,)*>
            where
                $($java_class_generics: duchess::JavaObject,)*
            {
                fn class<'jvm>(jvm: &mut Jvm<'jvm>) -> LocalResult<'jvm, Local<'jvm, java::lang::Class>> {
                    static CLASS: OnceCell<Java<java::lang::Class>> = OnceCell::new();
                    let global = CLASS.get_or_try_init::<_, duchess::Error<Local<java::lang::Throwable>>>(|| {
                        let class = duchess::plumbing::find_class(jvm, $jni_class_name)?;
                        Ok(jvm.global(&class))
                    })?;
                    Ok(jvm.local(global))
                }
            }

            impl<$($java_class_generics,)*> ::core::convert::AsRef<$struct_name<$($java_class_generics,)*>> for $struct_name<$($java_class_generics,)*>
            where
                $($java_class_generics: duchess::JavaObject,)*
            {
                fn as_ref(&self) -> &$struct_name<$($java_class_generics,)*> {
                    self
                }
            }

            impl<$($java_class_generics,)*> ::core::ops::Deref for $struct_name<$($java_class_generics,)*>
            where
                $($java_class_generics: duchess::JavaObject,)*
            {
                type Target = <Self as duchess::plumbing::JavaView>::OfObj<Self>;

                fn deref(&self) -> &Self::Target {
                    duchess::plumbing::FromRef::from_ref(self)
                }
            }

            impl<$($java_class_generics,)*> duchess::prelude::JDeref for $struct_name<$($java_class_generics,)*>
            where
                $($java_class_generics: duchess::JavaObject,)*
            {
                fn jderef(&self) -> &Self {
                    self
                }
            }

            impl<$($java_class_generics,)*> duchess::prelude::TryJDeref for $struct_name<$($java_class_generics,)*>
            where
                $($java_class_generics: duchess::JavaObject,)*
            {
                type Java = Self;

                fn try_jderef(&self) -> duchess::Nullable<&Self> {
                    Ok(self)
                }
            }

            // Reflexive upcast impl
            unsafe impl<$($java_class_generics,)*> duchess::plumbing::Upcast<$struct_name<$($java_class_generics,)*>> for $struct_name<$($java_class_generics,)*>
            where
                $($java_class_generics: duchess::JavaObject,)*
            {}

            duchess::plumbing::setup_class! {
                @upcast_impls($struct_name, [$($mro_ty,)*], [$($java_class_generics,)*])
            }

            impl<$($java_class_generics,)* > $struct_name<$($java_class_generics,)*>
            where
                $($java_class_generics: duchess::JavaObject,)*
            {
                $($constructors)*

                $($static_methods)*

                $($static_field_getters)*

                $($inherent_object_methods)*
            }

            impl<$($java_class_generics,)*> duchess::plumbing::JavaView for $struct_name<$($java_class_generics,)*>
            {
                type OfOp<J> = $op_name<
                    $($java_class_generics,)* J,
                    mro!(J, OfOpWith, [$($mro_ty,)*])
                >;

                type OfOpWith<J, N> = $op_name<
                    $($java_class_generics,)* J,
                    N,
                >
                where
                    N: duchess::plumbing::FromRef<J>;

                type OfObj<J> = $obj_name<
                    $($java_class_generics,)* J,
                    mro!(J, OfObjWith, [$($mro_ty,)*])
                >;

                type OfObjWith<J, N> = $obj_name<
                    $($java_class_generics,)* J,
                    N,
                >
                where
                    N: duchess::plumbing::FromRef<J>;
            }

            impl<$($java_class_generics,)* J, N> $op_name<$($java_class_generics,)* J, N>
            where
                $($java_class_generics: duchess::JavaObject,)*
                J: duchess::plumbing::JvmRefOp<$struct_name<$($java_class_generics,)*>>,
                N: duchess::plumbing::FromRef<J>,
            {
                $($op_struct_methods)*
            }

            impl<$($java_class_generics,)* J, N> $obj_name<$($java_class_generics,)* J, N>
            where
                $($java_class_generics: duchess::JavaObject,)*
                for<'jvm> &'jvm J: duchess::plumbing::JvmRefOp<$struct_name<$($java_class_generics,)*>>,
            {
                $($obj_struct_methods)*
            }

            duchess::plumbing::setup_class! {
                @op_obj_definitions($struct_name, $op_name, [$($java_class_generics,)*])
            }

            duchess::plumbing::setup_class! {
                @op_obj_definitions($struct_name, $obj_name, [$($java_class_generics,)*])
            }
        };
    };

    (@op_obj_definitions($struct_name:ident, $opobj_name:ident, [$($java_class_generics:ident,)*])) => {
        duchess::plumbing::setup_class! {
            @op_obj_struct($struct_name, $opobj_name, [$($java_class_generics,)*])
        }

        duchess::plumbing::setup_class! {
            @op_obj_FromRef_impl($opobj_name, [$($java_class_generics,)*])
        }

        duchess::plumbing::setup_class! {
            @op_obj_Deref_impl($opobj_name, [$($java_class_generics,)*])
        }
    };

    (@op_obj_struct($struct_name:ident, $opobj_name:ident, [$($java_class_generics:ident,)*])) => {
        #[repr(transparent)]
        pub struct $opobj_name<$($java_class_generics,)* J, N> {
            this: J,
            phantom: ::core::marker::PhantomData<($struct_name<$($java_class_generics,)*>, N)>,
        }
    };

    (@op_obj_Deref_impl($opobj_name:ident,[$($java_class_generics:ident,)*])) => {
        impl<$($java_class_generics,)* J, N> ::core::ops::Deref for $opobj_name<$($java_class_generics,)* J, N>
        where
            N: duchess::plumbing::FromRef<J>,
        {
            type Target = N;

            fn deref(&self) -> &N {
                duchess::plumbing::FromRef::from_ref(&self.this)
            }
        }
    };

    (@op_obj_FromRef_impl($opobj_name:ident, [$($java_class_generics:ident,)*])) => {
        impl<$($java_class_generics,)* J, N> duchess::plumbing::FromRef<J> for $opobj_name<$($java_class_generics,)* J, N> {
            fn from_ref(j: &J) -> &Self {
                // This is safe because of the `#[repr(transparent)]`
                // on the struct declaration.
                unsafe {
                    ::core::mem::transmute::<&J, &Self>(j)
                }
            }
        }
    };

    // Create an `Upcast` impl for each type in to the MRO.
    // The obvious pattern to do this [look like this][pg]
    // but for some reason that gives an error in macro-by-rules expansion,
    // so we use a recursive macro-rules invocation.
    //
    // [pg]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=6bdda5a7268d02833cb1730d5558723a

    (@upcast_impls($struct_name:ident, [], [$($java_class_generics:ident,)*])) => {};
    (@upcast_impls($struct_name:ident, [$mro_head_ty:ty, $($mro_tail_ty:ty,)*], [$($java_class_generics:ident,)*])) => {
        unsafe impl<$($java_class_generics,)*> duchess::plumbing::Upcast<$mro_head_ty> for $struct_name<$($java_class_generics,)*>
        where
            $($java_class_generics: duchess::JavaObject,)*
        {}

        duchess::plumbing::setup_class! {
            @upcast_impls($struct_name, [$($mro_tail_ty,)*], [$($java_class_generics,)*])
        }
    };
}
