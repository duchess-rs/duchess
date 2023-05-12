use crate::{
    argument::DuchessDeclaration,
    class_info::{
        ClassInfo, ClassRef, Constructor, DotId, Field, Id, Method, NonRepeatingType, RootMap, SpannedPackageInfo, Type,
    },
    reflect::Reflector,
    signature::Signature,
    span_error::SpanError,
};
use inflector::Inflector;
use once_cell::sync::OnceCell;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote_spanned;
use rust_format::Formatter;

impl DuchessDeclaration {
    pub fn to_tokens(&self) -> Result<TokenStream, SpanError> {
        let reflector = &mut Reflector::default();
        let root_map = self.to_root_map(reflector)?;
        let () = root_map.check(reflector)?;
        root_map.to_tokens(reflector)
    }
}

impl RootMap {
    fn to_tokens(self, reflector: &mut Reflector) -> Result<TokenStream, SpanError> {
        self.to_packages()
            .map(|p| p.to_tokens(&[], &self, reflector))
            .collect()
    }
}

/// The various pieces that we use to reflect a Java method into Rust.
struct MethodOutput {
    /// Declaration of the struct for the method, e.g., `struct toString<...> { ... }`.
    method_struct: TokenStream,

    /// Declaration of the items for the method in the `FooExt` trait, e.g.
    /// `type toString: IntoOptionLocal<String>; fn toString(&self) -> Self::toString;`
    trait_method: TokenStream,

    /// Declaration of the `type` and `fn` to be used in the blanket impl we are going to create,
    /// which will map the associated type to the `method_struct`.
    trait_impl_method: TokenStream,

    /// Implementation of `jvmop` for the method struct.
    jvm_op_impl: TokenStream,
}

impl SpannedPackageInfo {
    fn to_tokens(
        &self,
        parents: &[Id],
        root_map: &RootMap,
        reflector: &mut Reflector,
    ) -> Result<TokenStream, SpanError> {
        let package_id = DotId::new(parents, &self.name);
        let name = self.name.to_ident(self.span);

        let subpackage_tokens: TokenStream = self
            .subpackages
            .values()
            .map(|p| p.to_tokens(&package_id, root_map, reflector))
            .collect::<Result<_, _>>()?;

        let class_tokens: TokenStream = self
            .classes
            .iter()
            .map(|class_id| root_map.classes[class_id].to_tokens())
            .collect::<Result<_, _>>()?;

        let supers: Vec<TokenStream> = package_id
            .iter()
            .map(|_| quote_spanned!(self.span => super))
            .collect();

        Ok(quote_spanned!(self.span =>
            #[allow(unused_imports)]
            pub mod #name {
                // Import the contents of the parent module that we are created inside
                use #(#supers ::)* *;

                // Import the java package provided by duchess
                use duchess::java;

                #subpackage_tokens
                #class_tokens
            }
        ))
    }
}

impl ClassInfo {
    pub fn to_tokens(&self) -> Result<TokenStream, SpanError> {
        let struct_name = self.struct_name();
        let ext_trait_name = self.ext_trait_name();
        let cached_class = self.cached_class();
        let this_ty = self.this_type();
        let java_class_generics_with_defaults = self.class_generic_names_with_defaults();
        let java_class_generics = self.class_generic_names();

        // Convert constructors
        let constructors: Vec<_> = self
            .constructors
            .iter()
            .map(|c| self.constructor(c))
            .collect::<Result<_, _>>()?;

        // Convert static methods (not instance methods, those are different)
        let static_methods: Vec<_> = self
            .methods
            .iter()
            .filter(|m| m.flags.is_static)
            .map(|m| self.static_method(m))
            .collect::<Result<_, _>>()?;

        // Convert instance methods (not static methods, those are different)
        let object_methods: Vec<_> = self
            .methods
            .iter()
            .filter(|m| !m.flags.is_static)
            .map(|m| self.object_method(m))
            .collect::<Result<_, _>>()?;

        // Generate static field getters
        let static_field_getters: Vec<_> = self
            .fields
            .iter()
            .filter(|f| f.flags.is_static)
            .map(|f| self.static_field_getter(f))
            .collect::<Result<_, _>>()?;

        let method_structs: Vec<_> = object_methods.iter().map(|m| &m.method_struct).collect();
        let trait_methods: Vec<_> = object_methods.iter().map(|m| &m.trait_method).collect();
        let trait_impl_methods: Vec<_> = object_methods
            .iter()
            .map(|m| &m.trait_impl_method)
            .collect();
        let jvm_op_impls: Vec<_> = object_methods.iter().map(|m| &m.jvm_op_impl).collect();
        let upcast_impls = self.upcast_impls()?;

        let output = quote_spanned! {
            self.span =>

            #[allow(non_camel_case_types)]
            pub struct #struct_name<#(#java_class_generics_with_defaults,)*> {
                _dummy: std::marker::PhantomData<(#(#java_class_generics,)*)>
            }

            #[allow(non_camel_case_types)]
            pub trait #ext_trait_name<#(#java_class_generics,)*> : duchess::JvmOp
            where
                #(#java_class_generics : duchess::JavaObject,)*
            {
                #(#trait_methods)*
            }

            // Hide other generated items
            #[allow(unused_imports)]
            const _: () = {
                use duchess::{
                    *,
                    codegen_deps::once_cell::sync::OnceCell,
                    plumbing::*,
                    prelude::*,
                };

                unsafe impl<#(#java_class_generics,)*> duchess::JavaObject for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    #cached_class
                }

                unsafe impl<#(#java_class_generics,)*> plumbing::Upcast<#struct_name<#(#java_class_generics,)*>> for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {}

                impl<#(#java_class_generics,)*> AsRef<#struct_name<#(#java_class_generics,)*>> for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    fn as_ref(&self) -> &#struct_name<#(#java_class_generics,)*> {
                        self
                    }
                }

                impl<#(#java_class_generics,)*> JDeref for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    fn jderef(&self) -> &Self {
                        self
                    }
                }

                impl<#(#java_class_generics,)*> TryJDeref for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    type Java = Self;

                    fn try_jderef(&self) -> Nullable<&Self> {
                        Ok(self)
                    }
                }

                #upcast_impls

                impl< #(#java_class_generics,)* > #this_ty
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    #(#constructors)*

                    #(#static_methods)*

                    #(#static_field_getters)*
                }

                #(#method_structs)*

                #(#jvm_op_impls)*

                #[allow(non_camel_case_types)]
                impl<This, #(#java_class_generics,)*> #ext_trait_name<#(#java_class_generics,)*> for This
                where
                    This: JvmOp,
                    for<'jvm> This::Output<'jvm>: duchess::AsJRef<#this_ty>,
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    #(#trait_impl_methods)*
                }
            };
        };

        if let Ok(f) = std::env::var("DUCHESS_DEBUG") {
            if f == "*" || f == "1" || self.name.to_string().starts_with(&f) {
                match rust_format::RustFmt::default().format_tokens(output.clone()) {
                    Ok(v) => {
                        eprintln!("{v}");
                    }
                    Err(_) => {
                        eprintln!("{output:?}");
                    }
                }
            }
        }

        Ok(output)
    }

    fn upcast_impls(&self) -> Result<TokenStream, SpanError> {
        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();
        self
            .resolve_upcasts()
            .map(|r| {
                let mut sig = Signature::new(&Id::from("supertrait"), self.span, &[])
                .with_internal_generics(&self.generics)?;
                let tokens = sig.forbid_capture(|sig| sig.class_ref_ty(r)).unwrap();
                Ok(quote_spanned!(self.span =>
                    unsafe impl<#(#java_class_generics,)*> plumbing::Upcast<#tokens> for #struct_name<#(#java_class_generics,)*>
                    where
                        #(#java_class_generics: duchess::JavaObject,)*
                    {}
                ))
            })
            .collect()
    }

    // XX: Clearly, we'll need more sophisticated resolution of what types we descend from, but for now we can at least
    // inject the "everything is an Object" root.
    fn resolve_upcasts(&self) -> impl Iterator<Item = &'_ ClassRef> {
        static OBJECT: OnceCell<ClassRef> = OnceCell::new();
        let object = OBJECT.get_or_init(|| ClassRef {
            name: DotId::parse("java.lang.Object"),
            generics: vec![],
        });

        self.extends
            .iter()
            .chain(&self.implements)
            .chain(Some(object).filter(|obj| obj.name != self.name).into_iter())
    }

    fn cached_class(&self) -> TokenStream {
        let jni_class_name = self.jni_class_name();

        quote_spanned! {
            self.span =>
            fn class<'jvm>(jvm: &mut Jvm<'jvm>) -> duchess::Result<'jvm, Local<'jvm, java::lang::Class>> {
                static CLASS: OnceCell<Global<java::lang::Class>> = OnceCell::new();
                let global = CLASS.get_or_try_init::<_, duchess::Error<Local<java::lang::Throwable>>>(|| {
                    let class = find_class(jvm, #jni_class_name)?;
                    Ok(jvm.global(&class))
                })?;
                Ok(jvm.local(global))
            }
        }
    }

    fn constructor(&self, constructor: &Constructor) -> Result<TokenStream, SpanError> {
        let mut sig = Signature::new(self.name.class_name(), self.span, &self.generics);

        let input_traits: Vec<_> = constructor
            .argument_tys
            .iter()
            .map(|ty| sig.input_trait(ty))
            .collect::<Result<_, _>>()?;

        let input_names: Vec<_> = (0..input_traits.len())
            .map(|i| Ident::new(&format!("a{i}"), self.span))
            .collect();

        let ty = self.this_type();
        let output_trait = quote_spanned!(self.span => IntoLocal<#ty>);

        let java_class_generics = self.class_generic_names();

        let jni_descriptor = jni_c_str(constructor.descriptor(), self.span);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &constructor.argument_tys);

        // for debugging JVM invocation failures
        let name = Literal::string(&self.name.to_string());
        let descriptor = Literal::string(&constructor.descriptor());

        let output = quote_spanned!(self.span =>
            pub fn new(
                #(#input_names : impl #input_traits,)*
            ) -> impl #output_trait {
                #[allow(non_camel_case_types)]
                struct Impl<
                    #(#java_class_generics,)*
                    #(#input_names),*
                > {
                    #(#input_names: #input_names,)*
                    phantom: std::marker::PhantomData<(
                        #(#java_class_generics,)*
                    )>,
                }

                #[allow(non_camel_case_types)]
                impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                > JvmOp for Impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                >
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                    #(#input_names : #input_traits,)*
                {
                    type Output<'jvm> = Local<'jvm, #ty>;

                    fn execute_with<'jvm>(
                        self,
                        jvm: &mut Jvm<'jvm>,
                    ) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                        #(#prepare_inputs)*

                            let class = <#ty>::class(jvm)?;

                            // Cache the method id for the constructor -- note that we only have one cache
                            // no matter how many generic monomorphizations there are. This makes sense
                            // given Java's erased-based generics system.
                            static CONSTRUCTOR: OnceCell<MethodPtr> = OnceCell::new();
                            let constructor = CONSTRUCTOR.get_or_try_init(|| {
                                find_constructor(jvm, &class, #jni_descriptor)
                            })?;

                            let env = jvm.env();
                            let obj = unsafe {
                                env.invoke(|env| env.NewObjectA, |env, f| f(
                                    env,
                                    class.as_raw().as_ptr(),
                                    constructor.as_ptr(),
                                    [
                                        #(#input_names.into_jni_value(),)*
                                    ].as_ptr(),
                                ))
                            };

                            if let Some(obj) = ObjectPtr::new(obj) {
                                Ok(unsafe { Local::from_raw(env, obj) })
                            } else {
                                check_exception(jvm)?;
                                // NewObjectA should only return a null pointer when an exception occurred in the
                                // constructor, so reaching here is a strange JVM state
                                Err(duchess::Error::JvmInternal(format!(
                                    "failed to create new `{}` via constructor `{}`",
                                    #name, #descriptor,
                                )))
                            }
                        }
                    }

                Impl {
                    #(#input_names: #input_names,)*
                    phantom: Default::default()
                }
            }
        );

        // useful for debugging
        // eprintln!("{output}");

        Ok(output)
    }

    /// Generates code for instance methods.
    ///
    ///
    /// NB. This function (particularly the JvmOp impl) has significant overlap with `static_method`
    /// and `static_field_getter`, so if you make changes here, you may well need changes there.
    fn object_method(&self, method: &Method) -> Result<MethodOutput, SpanError> {
        assert!(!method.flags.is_static);

        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let this_ty = self.this_type();

        let input_traits: Vec<_> = method
            .argument_tys
            .iter()
            .map(|ty| sig.input_trait(ty))
            .collect::<Result<_, _>>()?;

        let input_names: Vec<_> = (0..input_traits.len())
            .map(|i| Ident::new(&format!("a{i}"), self.span))
            .collect();

        let output_ty = sig.output_type(&method.return_ty)?;
        let output_trait = sig.method_trait(&method.return_ty)?;
        let jni_call_fn = sig.jni_call_fn(&method.return_ty)?;

        let jni_descriptor = jni_c_str(&method.descriptor(), self.span);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &method.argument_tys);

        let jni_method = jni_c_str(&*method.name, self.span);

        let rust_method_name = Id::from(method.name.to_snake_case()).to_ident(self.span);
        let rust_method_type_name = Id::from(method.name.to_camel_case()).to_ident(self.span);

        // The generic parameters declared on the Java method.
        let java_class_generics: Vec<_> = self.class_generic_names();

        // The generic parameters we need on the Rust method, these include:
        //
        // * a type parameter `a0` for each input
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics: Vec<_> = input_names.iter().chain(&sig.rust_generics).collect();

        // The generic parameters we need on the *method struct* (which will implement the `JvmOp`).
        // These include the class generics plus all the generics from the method.
        let method_struct_generics: Vec<_> = java_class_generics
            .iter()
            .chain(rust_method_generics.iter().copied())
            .collect();

        // For each method `m` in the Java type, we create a struct (named `m`)
        // that will implement the `JvmOp`.
        let method_struct = quote_spanned!(self.span =>
            #[derive(Clone)]
            #[allow(non_camel_case_types)]
            pub struct #rust_method_type_name<
                This,
                #(#method_struct_generics,)*
            > {
                this: This,
                #(#input_names : #input_names,)*
                phantom: std::marker::PhantomData<(
                    #(#method_struct_generics,)*
                )>,
            }
        );

        let sig_where_clauses = &sig.where_clauses;

        // The method signature for the extension trait.
        let trait_method = quote_spanned!(self.span =>
            type #rust_method_type_name<#(#rust_method_generics),*>: #output_trait
            where
                #(#input_names: #input_traits,)*
                #(#sig_where_clauses,)*
                ;

            fn #rust_method_name<#(#rust_method_generics),*>(
                self,
                #(#input_names: #input_names),*
            ) -> Self::#rust_method_type_name<#(#rust_method_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#sig_where_clauses,)*
                ;
        );

        // The method signature for the extension trait.
        let trait_impl_method = quote_spanned!(self.span =>
            type #rust_method_type_name<#(#rust_method_generics),*> =
                #rust_method_type_name<Self, #(#method_struct_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#sig_where_clauses,)*
                ;

            fn #rust_method_name<#(#rust_method_generics),*>(
                self,
                #(#input_names: #input_names),*
            ) -> Self::#rust_method_type_name<#(#rust_method_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#sig_where_clauses,)*
            {
                #rust_method_type_name {
                    this: self,
                    #(#input_names: #input_names,)*
                    phantom: Default::default(),
                }
            }
        );

        // Implementation of `JvmOp` for `m` -- when executed, call the method
        // via JNI, after converting its arguments appropriately.
        let impl_output = quote_spanned!(self.span =>
            #[allow(non_camel_case_types)]
            impl<This, #(#method_struct_generics),*> JvmOp
            for #rust_method_type_name<This, #(#method_struct_generics),*>
            where
                This: JvmOp,
                for<'jvm> This::Output<'jvm>: duchess::AsJRef<#this_ty>,
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                type Output<'jvm> = #output_ty;

                fn execute_with<'jvm>(
                    self,
                    jvm: &mut Jvm<'jvm>,
                ) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                    let this = self.this.execute_with(jvm)?;
                    let this: & #this_ty = this.as_jref()?;
                    let this = this.as_raw();

                    #(#prepare_inputs)*

                    // Cache the method id for this method -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static METHOD: OnceCell<MethodPtr> = OnceCell::new();
                    let method = METHOD.get_or_try_init(|| {
                        let class = <#this_ty>::class(jvm)?;
                        find_method(jvm, &class, #jni_method, #jni_descriptor, false)
                    })?;

                    let output = unsafe {
                        jvm.env().invoke(|env| env.#jni_call_fn, |env, f| f(
                            env,
                            this.as_ptr(),
                            method.as_ptr(),
                            [
                                #(#input_names.into_jni_value(),)*
                            ].as_ptr(),
                        ))
                    };
                    check_exception(jvm)?;

                    let output: #output_ty = unsafe { FromJniValue::from_jni_value(jvm, output) };
                    Ok(output)
                }
            }
        );

        // useful for debugging
        // eprintln!("{trait_method}");
        // eprintln!("{trait_impl_method}");

        Ok(MethodOutput {
            method_struct,
            trait_method,
            trait_impl_method,
            jvm_op_impl: impl_output,
        })
    }

    /// Generates a static method declaration that should be part of the inherent methods
    /// for the struct. Unlike instance methods, static methods can be totally self-contained.
    ///
    /// NB. This function (particularly the JvmOp impl) has significant overlap with `object_method`
    /// and `static_field_getter`, so if you make changes here, you may well need changes there.
    fn static_method(&self, method: &Method) -> Result<TokenStream, SpanError> {
        assert!(method.flags.is_static);

        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let input_traits: Vec<_> = method
            .argument_tys
            .iter()
            .map(|ty| sig.input_trait(ty))
            .collect::<Result<_, _>>()?;

        let input_names: Vec<_> = (0..input_traits.len())
            .map(|i| Ident::new(&format!("a{i}"), self.span))
            .collect();

        let output_ty = sig.output_type(&method.return_ty)?;
        let output_trait = sig.method_trait(&method.return_ty)?;
        let jni_call_fn = sig.jni_static_call_fn(&method.return_ty)?;

        let jni_descriptor = jni_c_str(&method.descriptor(), self.span);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &method.argument_tys);

        let jni_method = jni_c_str(&*method.name, self.span);

        let rust_method_name = Id::from(method.name.to_snake_case()).to_ident(self.span);
        let rust_method_type_name = Id::from(method.name.to_camel_case()).to_ident(self.span);

        // The generic parameters declared on the Java method.
        let java_class_generics: Vec<_> = self.class_generic_names();

        // The generic parameters we need on the Rust method, these include:
        //
        // * a type parameter `a0` for each input
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics: Vec<_> = input_names.iter().chain(&sig.rust_generics).collect();

        // The generic parameters we need on the *method struct* (which will implement the `JvmOp`).
        // These include the class generics plus all the generics from the method.
        let method_struct_generics: Vec<_> = java_class_generics
            .iter()
            .chain(rust_method_generics.iter().copied())
            .collect();

        // For each method `m` in the Java type, we create a struct (named `m`)
        // that will implement the `JvmOp`.
        let method_struct = quote_spanned!(self.span =>
            #[derive(Clone)]
            #[allow(non_camel_case_types)]
            pub struct #rust_method_type_name<
                #(#method_struct_generics,)*
            > {
                #(#input_names : #input_names,)*
                phantom: std::marker::PhantomData<(
                    #(#method_struct_generics,)*
                )>,
            }
        );

        let sig_where_clauses = &sig.where_clauses;

        // Implementation of `JvmOp` for `m` -- when executed, call the method
        // via JNI, after converting its arguments appropriately.
        let this_ty = self.this_type();
        let jvmop_impl = quote_spanned!(self.span =>
            #[allow(non_camel_case_types)]
            impl<#(#method_struct_generics),*> JvmOp
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                type Output<'jvm> = #output_ty;

                fn execute_with<'jvm>(
                    self,
                    jvm: &mut Jvm<'jvm>,
                ) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                    #(#prepare_inputs)*

                    // Cache the method id for this method -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static METHOD: OnceCell<MethodPtr> = OnceCell::new();
                    let method = METHOD.get_or_try_init(|| {
                        let class = <#this_ty>::class(jvm)?;
                        find_method(jvm, &class, #jni_method, #jni_descriptor, true)
                    })?;

                    let class = <#this_ty>::class(jvm)?;
                    let output = unsafe {
                        jvm.env().invoke(|env| env.#jni_call_fn, |env, f| f(
                            env,
                            class.as_raw().as_ptr(),
                            method.as_ptr(),
                            [
                                #(#input_names.into_jni_value(),)*
                            ].as_ptr(),
                        ))
                    };
                    check_exception(jvm)?;

                    let output: #output_ty = unsafe { FromJniValue::from_jni_value(jvm, output) };
                    Ok(output)
                }
            }
        );

        let inherent_method = quote_spanned!(self.span =>
            #[allow(non_camel_case_types)]
            pub fn #rust_method_name<#(#rust_method_generics),*>(
                #(#input_names: #input_names),*
            ) -> impl #output_trait
            where
                #(#input_names: #input_traits,)*
                #(#sig_where_clauses,)*
            {
                #method_struct

                #jvmop_impl

                #rust_method_type_name {
                    #(#input_names: #input_names,)*
                    phantom: Default::default(),
                }
            }
        );

        Ok(inherent_method)
    }

    /// Generates a static field getter that should be part of the inherent methods
    /// for the struct.
    ///
    /// NB. This function (particularly the JvmOp impl) has significant overlap with `object_method`
    /// and `static_method`, so if you make changes here, you may well need changes there.
    fn static_field_getter(&self, field: &Field) -> Result<TokenStream, SpanError> {
        assert!(field.flags.is_static);

        let mut sig = Signature::new(&field.name, self.span, &self.generics);

        let output_ty = sig.non_void_output_type(&field.ty)?;
        let output_trait = sig.field_trait(&field.ty)?;
        let jni_field_fn = sig.jni_static_field_get_fn(&field.ty)?;

        let jni_field = jni_c_str(&*field.name, self.span);
        let jni_descriptor = jni_c_str(&field.ty.descriptor(), self.span);

        let rust_field_name = Id::from(format!("get_{}", field.name.to_snake_case())).to_ident(self.span);
        let rust_field_type_name = Id::from(format!("{}Getter", field.name.to_camel_case())).to_ident(self.span);

        // The generic parameters declared on the Java method.
        let java_class_generics: Vec<_> = self.class_generic_names();

        // The generic parameters we need on the *method struct* (which will implement the `JvmOp`).
        // These include the class generics plus all the generics from the method.
        let field_struct_generics: Vec<_> = java_class_generics.clone(); // XX: Unnecessary clone

        // For each field `f` in the Java type, we create a struct (named `<f>Getter`)
        // that will implement the `JvmOp`.
        let field_struct = quote_spanned!(self.span =>
            #[derive(Clone)]
            #[allow(non_camel_case_types)]
            pub struct #rust_field_type_name<
                #(#field_struct_generics,)*
            > {
                phantom: std::marker::PhantomData<(
                    #(#field_struct_generics,)*
                )>,
            }
        );

        let sig_where_clauses = &sig.where_clauses;

        // Implementation of `JvmOp` for `f` -- when executed, call the method
        // via JNI, after converting its arguments appropriately.
        let this_ty = self.this_type();
        let jvmop_impl = quote_spanned!(self.span =>
            #[allow(non_camel_case_types)]
            impl<#(#field_struct_generics),*> JvmOp
            for #rust_field_type_name<#(#field_struct_generics),*>
            where
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                type Output<'jvm> = #output_ty;

                fn execute_with<'jvm>(
                    self,
                    jvm: &mut Jvm<'jvm>,
                ) -> duchess::Result<'jvm, Self::Output<'jvm>> {

                    // Cache the field id for this field -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static FIELD: OnceCell<FieldPtr> = OnceCell::new();
                    let field = FIELD.get_or_try_init(|| {
                        let class = <#this_ty>::class(jvm)?;
                        find_field(jvm, &class, #jni_field, #jni_descriptor, true)
                    })?;

                    let class = <#this_ty>::class(jvm)?;
                    let output = unsafe {
                        jvm.env().invoke(|env| env.#jni_field_fn, |env, f| f(
                            env,
                            class.as_raw().as_ptr(),
                            field.as_ptr(),
                        ))
                    };
                    check_exception(jvm)?;

                    let output: #output_ty = unsafe { FromJniValue::from_jni_value(jvm, output) };
                    Ok(output)
                }
            }
        );

        let inherent_method = quote_spanned!(self.span =>
            #[allow(non_camel_case_types)]
            pub fn #rust_field_name() -> impl #output_trait
            where
                #(#sig_where_clauses,)*
            {
                #field_struct

                #jvmop_impl

                #rust_field_type_name {
                    phantom: Default::default(),
                }
            }
        );

        // useful for debugging
        // eprintln!("{inherent_method}");

        Ok(inherent_method)
    }

    fn struct_name(&self) -> Ident {
        self.name.class_name().to_ident(self.span)
    }

    fn ext_trait_name(&self) -> Ident {
        let mut id = self.name.class_name().clone();
        id.data.push_str("Ext");
        id.to_ident(self.span)
    }

    fn class_generic_names(&self) -> Vec<Ident> {
        self.generics
            .iter()
            .map(|g| g.to_ident(self.span))
            .collect()
    }

    fn class_generic_names_with_defaults(&self) -> Vec<TokenStream> {
        self.class_generic_names()
            .into_iter()
            .map(|g| quote_spanned!(self.span => #g = java::lang::Object))
            .collect()
    }

    fn this_type(&self) -> TokenStream {
        let s = self.struct_name();
        if self.generics.is_empty() {
            quote_spanned!(self.span => #s)
        } else {
            let g: Vec<Ident> = self.class_generic_names();
            quote_spanned!(self.span => #s < #(#g),* >)
        }
    }

    /// Returns a class name with `/`, like `java/lang/Object` as a &CStr
    fn jni_class_name(&self) -> TokenStream {
        jni_c_str(self.name.to_jni_name(), self.span)
    }

    fn prepare_inputs(&self, input_names: &[Ident], input_types: &[Type]) -> Vec<TokenStream> {
        input_names
            .iter()
            .zip(input_types)
            .map(|(input_name, input_ty)| match input_ty.to_non_repeating() {
                NonRepeatingType::Scalar(_) => quote_spanned!(self.span =>
                    let #input_name = self.#input_name.execute_with(jvm)?;
                ),
                NonRepeatingType::Ref(_) => quote_spanned!(self.span =>
                    let #input_name = self.#input_name.into_java(jvm)?;
                    let #input_name = #input_name.as_jref()?;
                ),
            })
            .collect()
    }
}

trait GenericExt {
    fn to_where_clause(&self, span: Span) -> TokenStream;
}

fn jni_c_str(contents: impl Into<String>, span: Span) -> TokenStream {
    let mut contents = contents.into().into_bytes();
    // \0 isn't valid UTF-8, so don't need to check that contents doesn't contain interior nul bytes.
    contents.push(0);

    let byte_string = Literal::byte_string(&contents);
    quote_spanned!(span => unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(#byte_string) })
}
