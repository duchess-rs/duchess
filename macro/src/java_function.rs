use std::{iter::once, sync::Arc};

use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;

use crate::{
    argument::MethodSelector,
    class_info::{self, ClassInfo, Method, Type},
    reflect::{MethodIndex, Reflector},
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

    let mut reflector = Reflector::default();
    let (class_info, method_index) = reflected_method(&selector, &mut reflector)?;
    let driver = Driver {
        selector: &selector,
        class_info: &class_info,
        method_info: &class_info.methods[method_index],
        input: &input,
    };

    let java_fn_name = driver.java_name();
    let input_fn_name = &input.sig.ident;

    // The first 2 arguments we expected from Java are always
    // a JVM environment and a `this` (or `class`, if static) pointer.
    let (
        Argument {
            name: env_name,
            ty: env_ty,
        },
        Argument {
            name: this_name,
            ty: this_ty,
        },
    ) = driver.default_arguments()?;

    // The next set of arguments ("user arguments") are based on the declared
    // arguments types from the Java class definition.
    let user_arguments = driver.user_arguments()?;
    let user_argument_names: Vec<_> = user_arguments.iter().map(|ua| &ua.name).collect();
    let user_argument_tys: Vec<_> = user_arguments.iter().map(|ua| &ua.ty).collect();

    // The "rust arguments" vector contains Rust expressions that convert from the
    // `this_name` and `user_argument_names` into the Rust types declared on the decorated function.
    let rust_arguments = driver.rust_arguments(&this_name, &user_argument_names)?;

    // The "main body" of the call -- invoke the decorated function with the appropriate arguments.
    let rust_invocation = quote_spanned!(span =>
        #input_fn_name(
            #(#rust_arguments),*
        )
    );

    // Wrap that "main body" with whatever we need to convert the returned value back
    // to the return type Java expects (`return_ty` is the Rust representation of that type)
    let (return_ty, rust_invocation) = driver.return_ty_and_expr(rust_invocation, &env_name)?;

    let vis = &input.vis;

    let rust_this_ty = driver.convert_ty(&class_info.this_ref().into())?;
    let method_name_literal = Literal::string(&selector.method_name());
    let signature_literal = Literal::string(&driver.method_info.descriptor());

    let tokens = quote_spanned!(span =>
        // Declare a function with no-mangle linkage as expected by Java.
        // The function is declared inside a `const _` block so that it is not nameable from Rust code.
        #[allow(unused_variables, nonstandard_style)]
        const _: () = {
            #[no_mangle]
            fn #java_fn_name(
                #env_name: #env_ty,
                #this_name: #this_ty,
                #(#user_argument_names: #user_argument_tys,)*
            ) -> #return_ty {
                // Covers the calls to the two `duchess::plumbing` functions,
                // both of which assume they are being invoked from within a JNI
                // method invoked by JVM. This function is anonymous and not
                // callable otherwise (presuming user doesn't directly invoke it
                // thanks to the `#[no_mangle]` attribute, in which case I'd say they are
                // asking for a problem).
                //
                // **NB.** It's important that #rust_invocation does not contain any user-given
                // code. If it did, that code could do unsafe things.
                unsafe {
                    #rust_invocation
                }
            }

            impl duchess::plumbing::JavaFn for #input_fn_name {
                fn java_fn() -> duchess::plumbing::JavaFunction {
                    unsafe {
                        duchess::plumbing::JavaFunction::new(
                            #method_name_literal,
                            #signature_literal,
                            std::ptr::NonNull::new_unchecked(#java_fn_name as *mut ()),
                            <#rust_this_ty as duchess::JavaObject>::class,
                        )
                    }
                }
            }
        };

        // Create a dummy type to represent this function (uninstantiable)
        #[allow(non_camel_case_types)]
        #vis struct #input_fn_name { _private: ::core::convert::Infallible }

        // Include the input from the user unchanged.
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
    reflector: &mut Reflector,
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
    input: &'a syn::ItemFn,
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
        let class_name = self.selector.class_name();
        let (package, class) = class_name.split();
        let method_name = self.selector.method_name();
        let symbol_name: String = once("Java")
            .chain(package.iter().map(|id| &id[..]))
            .chain(once(&class[..]))
            .chain(once(&method_name[..]))
            .collect::<Vec<_>>()
            .join("_");
        syn::Ident::new(&symbol_name, self.selector.span())
    }

    fn default_arguments(&self) -> syn::Result<(Argument, Argument)> {
        let span = self.selector.span();

        let env_arg = Argument {
            name: syn::Ident::new("jni_env", span),
            ty: quote_spanned!(span => duchess::plumbing::EnvPtr<'_>),
        };

        let this_ty = if self.method_info.flags.is_static {
            quote_spanned!(span => duchess::plumbing::jni_sys::jclass)
        } else {
            let rust_this_ty = self.convert_ty(&self.class_info.this_ref().into())?;
            quote_spanned!(span => &#rust_this_ty)
        };

        let this_arg = Argument {
            name: syn::Ident::new("this", span),
            ty: this_ty,
        };

        Ok((env_arg, this_arg))
    }

    fn convert_ty(&self, ty: &Type) -> syn::Result<TokenStream> {
        Ok(Signature::new(
            &self.method_info.name,
            self.selector.span(),
            &self.class_info.generics,
        )
        .forbid_capture(|sig| sig.java_ty(ty))?)
    }

    fn user_arguments(&self) -> syn::Result<Vec<Argument>> {
        let span = self.selector.span();
        let mut arguments = vec![];

        for (argument_ty, index) in self.method_info.argument_tys.iter().zip(0..) {
            let name = syn::Ident::new(&format!("arg{index}"), span);

            let java_ty = self.convert_ty(argument_ty)?;
            let ty = match argument_ty {
                class_info::Type::Ref(_) | class_info::Type::Repeat(_) => {
                    quote_spanned!(span => &#java_ty)
                }

                class_info::Type::Scalar(_) => java_ty,
            };

            arguments.push(Argument { name, ty })
        }

        Ok(arguments)
    }

    fn rust_arguments(
        &self,
        this_name: &Ident,
        user_names: &[&Ident],
    ) -> syn::Result<Vec<TokenStream>> {
        // Extract the `syn::PatType` version of the inputs. Error if `&self` etc is used.
        let mut input_rust_arguments = vec![];
        for fn_arg in &self.input.sig.inputs {
            match fn_arg {
                syn::FnArg::Receiver(r) => {
                    return Err(syn::Error::new(
                        r.span(),
                        "Rust methods cannot be mapped to Java native functions",
                    ));
                }
                syn::FnArg::Typed(t) => {
                    input_rust_arguments.push(t);
                }
            }
        }

        // Check that we have the right number of arguments and give a useful error otherwise.
        let expected_num_rust_arguments = if self.method_info.flags.is_static {
            0
        } else {
            1 // we expect a `this` argument
        } + user_names.len();
        if input_rust_arguments.len() > expected_num_rust_arguments {
            let extra_span = input_rust_arguments[expected_num_rust_arguments].span();
            return Err(syn::Error::new(
                extra_span,
                &format!(
                    "extra argument(s) on Rust function, only {} argument(s) are expected",
                    expected_num_rust_arguments
                ),
            ));
        } else if input_rust_arguments.len() < expected_num_rust_arguments {
            // Heuristic: try to remind user about `this`
            if !self.method_info.flags.is_static && input_rust_arguments.len() == 0 {
                return Err(syn::Error::new(
                    self.input.sig.ident.span(),
                    &format!(
                        "Rust function should have {} argument(s); don't forget about `this`",
                        expected_num_rust_arguments
                    ),
                ));
            } else if !self.method_info.flags.is_static {
                return Err(syn::Error::new(
                    self.input.sig.ident.span(),
                    &format!(
                        "Rust function should have {} argument(s); don't forget about `this`",
                        expected_num_rust_arguments
                    ),
                ));
            } else {
                return Err(syn::Error::new(
                    self.input.sig.ident.span(),
                    &format!(
                        "Rust function should have {} argument(s)",
                        expected_num_rust_arguments
                    ),
                ));
            }
        }

        // Output accumulator
        let mut output = vec![];

        let mut inputs = input_rust_arguments.iter();

        // Push the `this` argument onto `output`
        if !self.method_info.flags.is_static {
            output.push(self.rust_argument(this_name, false, inputs.next().unwrap())?);
        }

        // Push each subsequent argument
        for (user_name, argument_ty) in user_names.iter().zip(&self.method_info.argument_tys) {
            output.push(self.rust_argument(
                user_name,
                argument_ty.is_scalar(),
                inputs.next().unwrap(),
            )?);
        }

        Ok(output)
    }

    fn rust_argument(
        &self,
        arg_name: &Ident,
        java_ty_is_scalar: bool,
        rust_ty: &syn::PatType,
    ) -> syn::Result<TokenStream> {
        // Case 1. Decorated Rust function has a `&J` type for this argument.
        // In that case, we provide the Java object unchanged.
        if let syn::Type::Reference(_) = &*rust_ty.ty {
            // If the decorated Rust function argument type is a Rust reference
            // (`&J`), then just pass the Java type directly.
            if java_ty_is_scalar {
                return Err(syn::Error::new(
                        rust_ty.ty.span(),
                        &format!("unexpected Rust reference; Java function declares a scalar type for this argument"),
                    ));
            }

            return Ok(quote_spanned!(rust_ty.span() => #arg_name));
        }

        // Case 2. Java type is scalar. Then just pass it.
        if java_ty_is_scalar {
            return Ok(quote_spanned!(rust_ty.span() => #arg_name));
        }

        // Case 3. Decorated Rust function has some Rust type; convert Java reference to that.
        Ok(quote_spanned!(rust_ty.span() => duchess::JvmOp::to_rust(#arg_name).execute()))
    }

    fn return_ty_and_expr(
        &self,
        return_expr: TokenStream,
        env_name: &Ident,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let span = self.selector.span();
        match &self.method_info.return_ty {
            Some(ty) => match ty {
                class_info::Type::Scalar(ty) => Ok((
                    ty.to_tokens(span),
                    quote_spanned!(span => duchess::plumbing::native_function_returning_scalar(#env_name, || #return_expr)),
                )),
                class_info::Type::Ref(_) | class_info::Type::Repeat(_) => {
                    let output_java_ty = self.convert_ty(ty)?;
                    Ok((
                        quote_spanned!(span => duchess::plumbing::jni_sys::jobject),
                        quote_spanned!(span => duchess::plumbing::native_function_returning_object::<#output_java_ty, _>(#env_name, || #return_expr)),
                    ))
                }
            },

            None => Ok((quote_spanned!(span => ()), return_expr)),
        }
    }
}
