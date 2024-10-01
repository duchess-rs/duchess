#[macro_export]
macro_rules! setup_class {
    (
        struct_name: [$S:ident],
        java_class_generics: [$($G:ident,)*],
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
        // Create the Rust struct $S that will represent the Java struct.
        // It is impossible to create an instance of this struct.

        #[allow(non_camel_case_types)]
        pub struct $S<$($G = duchess::java::lang::Object,)*> {
            _empty: std::convert::Infallible,
            _dummy: ::core::marker::PhantomData<($($G,)*)>
        }

        // Hide other generated items
        #[allow(unused_imports)]
        #[allow(nonstandard_style)]
        const _: () = {
            use duchess::{java, Java, Jvm, Local, LocalResult};
            use duchess::plumbing::once_cell::sync::OnceCell;
            use duchess::plumbing::mro;

            // Implement the `JavaObject` trait for `$S`.
            // This impl is unsafe because we are asserting that
            // it is impossible to create an instance of this struct
            // (this is true because the field contains an uninhabited type).
            // We are also asserting that we maintain the invariant that
            // users only access it by reference and that every such reference
            // comes from the JNI.

            unsafe impl<$($G,)*> duchess::JavaObject for $S<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
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

            // Add a reflexive `AsRef` impl for `$S`.

            impl<$($G,)*> ::core::convert::AsRef<$S<$($G,)*>> for $S<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
            {
                fn as_ref(&self) -> &$S<$($G,)*> {
                    self
                }
            }

            // The TryJDeref trait indicates a type that can be dereferenced
            // to a reference to a java object (possibly returning null).
            // In our case, we deref to ourselves, and the operation is infallible.

            impl<$($G,)*> duchess::prelude::TryJDeref for $S<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
            {
                type Java = Self;

                fn try_jderef(&self) -> duchess::Nullable<&Self> {
                    Ok(self)
                }
            }

            // The JDeref trait refines `TryJDeref` for cases where the result cannot be null.

            impl<$($G,)*> duchess::prelude::JDeref for $S<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
            {
                fn jderef(&self) -> &Self {
                    self
                }
            }

            // The deref for `$S` derefs to a **view** onto `$S` based on the
            // method-resolution-order (MRO). This is a fairly complex topic.
            // See [the method chapter][mro] in the book for more details.
            //
            // [mro]: https://duchess-rs.github.io/duchess/methods.html

            impl<$($G,)*> ::core::ops::Deref for $S<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
            {
                type Target = <Self as duchess::plumbing::JavaView>::OfObj<Self>;

                fn deref(&self) -> &Self::Target {
                    duchess::plumbing::FromRef::from_ref(self)
                }
            }

            // Reflexive upcast impl

            unsafe impl<$($G,)*> duchess::plumbing::Upcast<$S<$($G,)*>> for $S<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
            {}

            // Generate impls that permit upcasting to every type on the [method-resolution-order][mro].
            // This is a recursive macro invocation to one of the "helper arms" below.
            //
            // [mro]: https://duchess-rs.github.io/duchess/methods.html

            duchess::plumbing::setup_class! {
                @upcast_impls($S, [$($mro_ty,)*], [$($G,)*])
            }

            // Add helper methods, constructors, and other things directly invokable on `$S`.

            impl<$($G,)* > $S<$($G,)*>
            where
                $($G: duchess::JavaObject,)*
            {
                $($constructors)*

                $($static_methods)*

                $($static_field_getters)*

                $($inherent_object_methods)*
            }

            // Helper structs for [managing method dispatch][mro]:
            //
            // * The `Op` struct, or "operation type", hosts methods that are available on the `JvmOp`
            // and produce another `JvmOp` (i.e., if you do `foo.bar().baz().execute()`,
            // the call to `baz()` is being invoked on a jvm-op representing the value that
            // will be returned by `bar` when execution actually occurs).
            //
            // * The `Op` struct, or "object type", hosts the same methods but is used
            // when invoking methods on an actual pointer to the object
            // (i.e., if you do `foo.bar()`, the `bar()` method
            // is being called on some reference to a Java object `foo`).
            //
            // [mro]: https://duchess-rs.github.io/duchess/methods.html

            duchess::plumbing::setup_class! {
                @op_obj_definitions($S, $op_name, [$($G,)*])
            }

            duchess::plumbing::setup_class! {
                @op_obj_definitions($S, $obj_name, [$($G,)*])
            }

            // Add the methods for the op/obj types, as explained above.

            impl<$($G,)* J, N> $op_name<$($G,)* J, N>
            where
                $($G: duchess::JavaObject,)*
                J: duchess::plumbing::JvmRefOp<$S<$($G,)*>>,
                N: duchess::plumbing::FromRef<J>,
            {
                $($op_struct_methods)*
            }

            impl<$($G,)* J, N> $obj_name<$($G,)* J, N>
            where
                $($G: duchess::JavaObject,)*
                for<'jvm> &'jvm J: duchess::plumbing::JvmRefOp<$S<$($G,)*>>,
            {
                $($obj_struct_methods)*
            }

            // The `JavaView` type navigates the [method-resolution-order][mro].
            //
            // The `OfOp` associated type indicates the starting "operation type" for `$S`;
            // the `OfOpWith` associated type indicates the "operation type" for the next
            // class in the method resolution order.
            //
            // The `OfObj` associated type indicates the starting "object type" for `$S`;
            // the `OfObjWith` associated type indicates the "object type" for the next
            // class in the method resolution order.
            //
            // [mro]: https://duchess-rs.github.io/duchess/methods.html

            impl<$($G,)*> duchess::plumbing::JavaView for $S<$($G,)*>
            {
                type OfOp<J> = $op_name<
                    $($G,)* J,
                    mro!(J, OfOpWith, [$($mro_ty,)*])
                >;

                type OfOpWith<J, N> = $op_name<
                    $($G,)* J,
                    N,
                >
                where
                    N: duchess::plumbing::FromRef<J>;

                type OfObj<J> = $obj_name<
                    $($G,)* J,
                    mro!(J, OfObjWith, [$($mro_ty,)*])
                >;

                type OfObjWith<J, N> = $obj_name<
                    $($G,)* J,
                    N,
                >
                where
                    N: duchess::plumbing::FromRef<J>;
            }
        };
    };

    // Generate the struct definition and necessary impls for the op or obj struct.

    (@op_obj_definitions($S:ident, $opobj_name:ident, [$($G:ident,)*])) => {
        duchess::plumbing::setup_class! {
            @op_obj_struct($S, $opobj_name, [$($G,)*])
        }

        duchess::plumbing::setup_class! {
            @op_obj_FromRef_impl($opobj_name, [$($G,)*])
        }

        duchess::plumbing::setup_class! {
            @op_obj_Deref_impl($opobj_name, [$($G,)*])
        }
    };

    // Generate the struct definition for the op or obj struct.
    //
    // This is basically a newtyped version of `J`, and the
    // `#[repr(transparent)]` is used to ensure that we can
    // transmute a `&J` reference into an `&OpStruct<J, ...>`
    // (resp. `&ObjStruct<J, ...>`) reference.

    (@op_obj_struct($S:ident, $opobj_name:ident, [$($G:ident,)*])) => {
        #[repr(transparent)]
        pub struct $opobj_name<$($G,)* J, N> {
            this: J,
            phantom: ::core::marker::PhantomData<($S<$($G,)*>, N)>,
        }
    };

    // Generate a `Deref` impl for the op or object struct;
    // the `Op` (resp. `Obj`) struct will deref to the
    // `Op` (resp. `Obj`) struct for the next
    // item in the method resolution order.

    (@op_obj_Deref_impl($opobj_name:ident,[$($G:ident,)*])) => {
        impl<$($G,)* J, N> ::core::ops::Deref for $opobj_name<$($G,)* J, N>
        where
            N: duchess::plumbing::FromRef<J>,
        {
            type Target = N;

            fn deref(&self) -> &N {
                duchess::plumbing::FromRef::from_ref(&self.this)
            }
        }
    };

    // Generate a `FromRef` impl allowing the op/obj struct
    // to be created from the original object reference.
    // This is safe because of the `#[repr(transparent)]`
    // on the op/obj struct definitions.

    (@op_obj_FromRef_impl($opobj_name:ident, [$($G:ident,)*])) => {
        impl<$($G,)* J, N> duchess::plumbing::FromRef<J> for $opobj_name<$($G,)* J, N> {
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

    (@upcast_impls($S:ident, [], [$($G:ident,)*])) => {};
    (@upcast_impls($S:ident, [$mro_head_ty:ty, $($mro_tail_ty:ty,)*], [$($G:ident,)*])) => {
        unsafe impl<$($G,)*> duchess::plumbing::Upcast<$mro_head_ty> for $S<$($G,)*>
        where
            $($G: duchess::JavaObject,)*
        {}

        duchess::plumbing::setup_class! {
            @upcast_impls($S, [$($mro_tail_ty,)*], [$($G,)*])
        }
    };
}
