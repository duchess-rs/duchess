#[macro_export]
macro_rules! setup_java_function {
    (
        vis: $vis:vis,

        // The name of the Rust function defined by the user
        input_fn_name: $input_fn_name:ident,

        // The mangled name of the Java function expectedby the Java linked
        java_fn_name: $java_fn_name:ident,

        // The type of the class this method is defined on as a Rust type (e.g., `java::lang::Object`).
        rust_owner_ty: $rust_owner_ty:ty,
        
        // The type of `this` we expect to come from Java.
        // For static methods, this is `duchess::semver_unstable::jni_sys::jclass`.
        // For instance methods, this will be the type of the Java object,
        // e.g., `&java::lang::String`.
        abi_this_ty: $abi_this_ty:ty,

        // Name we will use for the "this" argument given from Java (of type `$abi_this_ty`).
        abi_this_name: $abi_this_name:ident,

        // Argument we will supply as "this" to the Rust code (if any).
        // If this is a static method, then this is not present,
        // but otherwise it is the same as `$abi_this_name`.
        call_this_name: $($call_this_name:ident)?,

        // The names/types of arguments taken from the javap output.
        // These are the exact values coming in and not derived from what the user wrote.
        abi_argument_names: [$($abi_argument_names:ident),*],
        abi_argument_tys: [$($abi_argument_tys:ty),*],

        // The return type expected by the C code (e.g., `jobject` or `i32`)
        abi_return_ty: $abi_return_ty:ty,

        // The return type taken from `javap` and 
        rust_return_ty: $rust_return_ty:ty,

        // The appropriate function from `semver_unstable` to call.
        native_function_returning: $native_function_returning:ident,

        // The name of the method as a string literal.
        method_name_literal: $method_name_literal:expr,

        // JNI signature.
        signature_literal: $signature_literal:expr,
    ) => {
        // Declare a function with no-mangle linkage as expected by Java.
        // The function is declared inside a `const _` block so that it is not nameable from Rust code.
        #[allow(unused_variables, nonstandard_style)]
        const _: () = {
            #[no_mangle]
            extern "C" fn $java_fn_name(
                env: duchess::semver_unstable::EnvPtr<'_>,
                $abi_this_name: $abi_this_ty,
                $($abi_argument_names: $abi_argument_tys,)*
            ) -> $abi_return_ty {
                // Covers the calls to the two `duchess::plumbing` functions,
                // both of which assume they are being invoked from within a JNI
                // method invoked by JVM. This function is anonymous and not
                // callable otherwise (presuming user doesn't directly invoke it
                // thanks to the `#[no_mangle]` attribute, in which case I'd say they are
                // asking for a problem).
                unsafe {
                    duchess::semver_unstable::$native_function_returning::<
                        $rust_return_ty,
                        _,
                    >(
                        env,
                        || $input_fn_name(
                            $($call_this_name,)*
                            $($abi_argument_names,)*
                        )
                    )
                }
            }

            impl duchess::semver_unstable::JavaFn for $input_fn_name {
                fn java_fn() -> duchess::semver_unstable::JavaFunction {
                    unsafe {
                        duchess::semver_unstable::JavaFunction::new(
                            $method_name_literal,
                            $signature_literal,
                            std::ptr::NonNull::new_unchecked($java_fn_name as *mut ()),
                            <$rust_owner_ty as duchess::JavaObject>::class,
                        )
                    }
                }
            }
        };

        // Create a dummy type to represent this function (uninstantiable)
        #[allow(non_camel_case_types)]
        $vis struct $input_fn_name { _private: ::core::convert::Infallible }
    }
}