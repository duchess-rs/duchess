use std::{iter::once, sync::Arc};

use duchess_reflect::{class_info::ClassInfoAccessors, reflect::PrecomputedReflector};
use proc_macro2::{ Literal, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;

use crate::{
    argument::MethodSelector,
    class_info::{self, ClassInfo, Method, Type},
    reflect::MethodIndex,
    signature::Signature,
};

/// Decorator applied to Rust functions that implement Java native methods.
///
/// # Specifying the function being defined
///
/// The `#[java_function(X)]` takes an argument `X` that specifies which Java function is being defined.
/// This argument `X` can have the following forms:
///
/// * `java.class.Name::method`, identifying a `native` method `method` defined in the class `java.class.Name`. There must be exactly one native method with the given name.
/// * a partial class definition like `class java.class.Name { native void method(int i); }` which identifies the method name along with its complete signature. This class definition must contain exactly one method as its member, and the types must match what is declared in the Java class.
///
/// # Function arguments
///
/// The Rust function being decorated should have arguments matching the arguments to the Java method:
///
/// * If the Java function expects a scalar, the corresponding Rust scalar type is required (e.g., a Java `int` requires a Rust `i32`).
/// * If the Java function expects a Java object of type `J`, the Rust function should either expect
///     * a Rust reference `&J` to the Java object
///     * a Rust value of some type `R` which can be produced from `J` via the `JvmOp::to_rust` method.
///
/// # More details
///
/// See the [Duchess book](https://duchess-rs.github.io/duchess/java_function.html) for more details.
pub fn java_function(selector: MethodSelector, input: syn::ItemFn) -> syn::Result<TokenStream> {
    let span = selector.span();

    let reflector =
        PrecomputedReflector::new().map_err(|err| syn::Error::new(span, format!("{:?}", err)))?;
    let (class_info, method_index) = reflected_method(&selector, &reflector)?;
    let driver = Driver {
        selector: &selector,
        class_info: &class_info,
        method_info: &class_info.methods[method_index],
    };

    let vis = &input.vis;
    let java_fn_name = driver.java_name();
    let input_fn_name = &input.sig.ident;

    // The Rust type of the class this method is defined on.
    let rust_owner_ty = driver.convert_ty(&class_info.this_ref().into())?;

    // For instance methods, JNI provides the Java object reference, we type this as `&java::lang::String`
    // or whatever. Note that it cannot be null. We then pass this to the Rust function.
    //
    // For static methods, JNI provides a "this" argument whose value is the jclass.
    // We do not pass this to the Rust user and hence `call_this_name` is `None`.
    let abi_this_name = syn::Ident::new("this", span);
    let (abi_this_ty, call_this_name);
    if !driver.method_info.flags.is_static {
        abi_this_ty = quote_spanned!(span => &#rust_owner_ty);
        call_this_name = vec![&abi_this_name];
    } else {
        abi_this_ty = quote_spanned!(span => duchess::semver_unstable::jni_sys::jclass);
        call_this_name = vec![];
    };

    // Assemble the arguments that will be given by JNI code.
    let user_arguments = driver.user_arguments(&input)?;
    let abi_argument_names: Vec<_> = user_arguments.iter().map(|ua| &ua.name).collect();
    let abi_argument_tys: Vec<_> = user_arguments.iter().map(|ua| &ua.ty).collect();

    // Literals for giving to JNI.
    let method_name_literal = Literal::string(&selector.method_name());
    let signature_literal = Literal::string(
        &driver
            .method_info
            .descriptor(&class_info.as_ref().generics_scope()),
    );

    // Return types...
    let abi_return_ty; // ...that JNI expects
    let rust_return_ty ; // ...that Rust code should provide
    let native_function_returning;  // ...and the function that converts into the former from the latter
    match &driver.method_info.return_ty {
        Some(class_info::Type::Scalar(ty)) => {
            let output_rust_ty = ty.to_tokens(span);
            rust_return_ty = quote_spanned!(span => #output_rust_ty);
            abi_return_ty = quote_spanned!(span => #output_rust_ty);
            native_function_returning = quote_spanned!(span => native_function_returning_scalar);
        }
        Some(ty @ class_info::Type::Ref(_)) | Some(ty @ class_info::Type::Repeat(_)) => {
            rust_return_ty = driver.convert_ty(ty)?;
            abi_return_ty = quote_spanned!(span => duchess::semver_unstable::jni_sys::jobject);
            native_function_returning = quote_spanned!(span => native_function_returning_object);
        }
        None => {
            rust_return_ty = quote_spanned!(span => ());
            abi_return_ty = quote_spanned!(span => ());
            native_function_returning = quote_spanned!(span => native_function_returning_unit);
        }
    }

    let tokens = quote_spanned!(span =>
        duchess::semver_unstable::setup_java_function! {
            vis: #vis,
            input_fn_name: #input_fn_name,
            java_fn_name: #java_fn_name,
            rust_owner_ty: #rust_owner_ty,
            abi_this_ty: #abi_this_ty,
            abi_this_name: #abi_this_name,
            call_this_name: #(#call_this_name)*,
            abi_argument_names: [#(#abi_argument_names),*],
            abi_argument_tys: [#(#abi_argument_tys),*],
            abi_return_ty: #abi_return_ty,
            rust_return_ty: #rust_return_ty,
            native_function_returning: #native_function_returning,
            method_name_literal: #method_name_literal,
            signature_literal: #signature_literal,
        }

        #input
    );

    crate::debug_tokens(
        format!("{}::{}", selector.class_name(), selector.method_name()),
        &tokens,
    );

    Ok(tokens)
}

fn reflected_method(
    selector: &MethodSelector,
    reflector: &PrecomputedReflector,
) -> syn::Result<(Arc<ClassInfo>, MethodIndex)> {
    let reflected_method = reflector.reflect_method(selector)?;

    match reflected_method {
        crate::reflect::ReflectedMethod::Constructor(_, _) => Err(syn::Error::new(
            selector.span(),
            format!("cannot have a native class constructor"),
        )),
        crate::reflect::ReflectedMethod::Method(class_info, index) => Ok((class_info, index)),
    }
}

struct Driver<'a> {
    selector: &'a MethodSelector,
    class_info: &'a ClassInfo,
    method_info: &'a Method,
}

struct Argument {
    name: syn::Ident,
    ty: TokenStream,
}

impl Driver<'_> {
    /// Returns the name of the function that Java expects.
    fn java_name(&self) -> syn::Ident {
        // FIXME. This code is incomplete. See the rules here:
        //
        // https://docs.oracle.com/en/java/javase/12/docs/specs/jni/design.html#resolving-native-method-names
        //
        // We need to account for:
        //
        // * If the native method is overloaded (but only if it is), then the symbol name should include
        //   the descriptor.
        //
        // In the JNI format design document, there are 3 escape characters. _1, _2, and _3
        // https://docs.oracle.com/en/java/javase/17/docs/specs/jni/design.html
        //
        // Duchess currently supports _1, to escape underscores that appear in the class name
        // or in the package name. _2 and _3 are exclusively used to convey type information
        // for overloaded functions and are not needed at this time.
        //
        // Below are some examples of native functions that would require _2 and _3 escape characters
        // ```
        // // JavaCanCallRustJavaFunction.java
        // package test;
        //
        // public class JavaCanCallRustJavaFunction {
        //     public static native String baseGreeting(String name);
        //     public static native String baseGreeting(String[] name);
        //     public static native String baseGreeting(Object name);
        // }
        // ```
        //
        // The above java functions would map to the following C function signatures
        //
        // jstring Java_test_JavaCanCallRustJavaFunction_baseGreeting_f__ILjava_lang_String_2(jstring name) { ... }
        // jstring Java_test_JavaCanCallRustJavaFunction_baseGreeting_f__ILjava_lang_Object_2(jstring name) { ... }
        // jstring Java_test_JavaCanCallRustJavaFunction_baseGreeting_f__I_3java_lang_String_2(jstring name) { ... }
        let class_name = self.selector.class_name();
        let class = class_name.to_jni_class_name();
        let package = class_name.to_jni_package();
        let method_name = self.selector.method_name().replace("_", "_1");
        let symbol_name: String = once("Java")
            .chain(once(&package[..]))
            .chain(once(&class[..]))
            .chain(once(&method_name[..]))
            .collect::<Vec<_>>()
            .join("_");
        syn::Ident::new(&symbol_name, self.selector.span())
    }

    fn convert_ty(&self, ty: &Type) -> syn::Result<TokenStream> {
        Ok(Signature::new(
            &self.method_info.name,
            self.selector.span(),
            &self.class_info.generics,
        )
        .forbid_capture(|sig| sig.java_ty_rs(ty))?)
    }

    fn user_arguments(&self, input: &syn::ItemFn) -> syn::Result<Vec<Argument>> {
        let selector_span = self.selector.span();
        let mut arguments = vec![];

        let rust_offset = if self.method_info.flags.is_static { 0 } else { 1 };

        for (argument_ty, index) in self.method_info.argument_tys.iter().zip(0..) {
            // Try to get the span for the Nth argument from the Rust code.
            let arg_span = input.sig.inputs.iter().nth(index + rust_offset).map_or(selector_span, |arg| arg.span());

            let name = syn::Ident::new(&format!("arg{index}"), arg_span);

            let java_ty = self.convert_ty(argument_ty)?;
            let ty = match argument_ty {
                class_info::Type::Ref(_) | class_info::Type::Repeat(_) => {
                    quote_spanned!(arg_span => Option<&#java_ty>)
                }

                class_info::Type::Scalar(_) => java_ty,
            };

            arguments.push(Argument { name, ty })
        }

        Ok(arguments)
    }
}
