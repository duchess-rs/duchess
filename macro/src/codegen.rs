use crate::{
    argument::DuchessDeclaration,
    class_info::{
        ClassInfo, ClassRef, Constructor, Id, Method, RefType, ScalarType, SpannedClassInfo, Type,
    },
    span_error::SpanError,
};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote_spanned;
use rust_format::Formatter;

impl DuchessDeclaration {
    pub fn into_tokens(mut self) -> Result<TokenStream, SpanError> {
        todo!()
    }
}

struct MethodOutput {
    method_struct: TokenStream,
    trait_method: TokenStream,
    trait_impl_method: TokenStream,
    jvm_op_impl: TokenStream,
}

impl SpannedClassInfo {
    pub fn into_tokens(mut self) -> TokenStream {
        let struct_name = self.struct_name();
        let ext_trait_name = self.ext_trait_name();
        let cached_class = self.cached_class();
        let this_ty = self.this_type();
        let generics = self.generic_names();

        // Ignore constructors with unsupported wildcards.
        let constructors: Vec<_> = self
            .info
            .constructors
            .iter()
            .flat_map(|c| self.constructor(c).ok())
            .collect();

        // Ignore constructors with unsupported wildcards.
        let methods: Vec<_> = self
            .info
            .methods
            .iter()
            .flat_map(|m| self.method(m).ok())
            .collect();

        let method_structs: Vec<_> = methods.iter().map(|m| &m.method_struct).collect();
        let trait_methods: Vec<_> = methods.iter().map(|m| &m.trait_method).collect();
        let trait_impl_methods: Vec<_> = methods.iter().map(|m| &m.trait_impl_method).collect();
        let jvm_op_impls: Vec<_> = methods.iter().map(|m| &m.jvm_op_impl).collect();

        let output = quote_spanned! {
            self.span =>

            #[allow(non_camel_case_types)]
            pub struct #struct_name<#(#generics,)*> {
                _dummy: std::marker::PhantomData<(#(#generics,)*)>
            }

            #[allow(non_camel_case_types)]
            pub trait #ext_trait_name {
                #(#trait_methods)*
            }

            // Hide other generated items
            #[allow(unused_imports)]
            const _: () = {
                use duchess::{
                    *,
                    plumbing::*,
                    prelude::*,
                };
                use jni::{
                    objects::{AutoLocal, GlobalRef, JMethodID, JValueGen},
                    signature::ReturnType,
                    sys::jvalue,
                };
                use once_cell::sync::OnceCell;

                unsafe impl JavaObject for #struct_name {}

                unsafe impl plumbing::Upcast<#struct_name> for #struct_name {}

                #cached_class

                #(#constructors)*

                #(#method_structs)*

                #(#jvm_op_impls)*

                #[allow(non_camel_case_types)]
                impl<This, #(#generics,)*> #ext_trait_name for This
                where
                    This: JvmOp,
                    for<'jvm> This::Output<'jvm>: AsRef<#this_ty>,
                {
                    #(#trait_impl_methods)*
                }
            };
        };

        if std::env::var("DUCHESS_DEBUG").is_ok() {
            eprintln!(
                "{:?}",
                // output,
                rust_format::RustFmt::default().format_tokens(output.clone())
            );
        }

        output
    }

    fn cached_class(&self) -> TokenStream {
        let jni_class_name = self.jni_class_name();
        quote_spanned! {
            self.span =>

            fn cached_class(jvm: &mut Jvm<'_>) -> duchess::Result<&'static GlobalRef> {
                let env = jvm.to_env();

                static CLASS: OnceCell<GlobalRef> = OnceCell::new();
                CLASS.get_or_try_init(|| {
                    let class = env.find_class(#jni_class_name)?;
                    env.new_global_ref(class)
                })
            }
        }
    }

    fn constructor(&self, constructor: &Constructor) -> Result<TokenStream, UnsupportedWildcard> {
        let mut sig = Signature::new(self.span, &self.info.generics);

        let input_traits: Vec<_> = constructor
            .args
            .iter()
            .map(|ty| sig.input_trait(ty))
            .collect::<Result<_, _>>()?;

        let input_names: Vec<_> = (0..input_traits.len())
            .map(|i| Ident::new(&format!("a{i}"), self.span))
            .collect();

        let ty = self.this_type();
        let output_trait = quote_spanned!(self.span => IntoLocal<#ty>);

        let generics = self.generic_names();

        let descriptor = Literal::string(&constructor.descriptor.string);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &constructor.args);

        let output = quote_spanned!(self.span =>
            impl< #(#generics),* > #ty {
                pub fn new(
                    #(#input_names : impl #input_traits,)*
                ) -> impl #output_trait {
                    #[derive(Clone)]
                    #[allow(non_camel_case_types)]
                    struct Impl< #(#input_names),* > {
                        #(#input_names: #input_names),*
                    }

                    #[allow(non_camel_case_types)]
                    impl<#(#input_names),*> JvmOp for Impl<#(#input_names),*>
                    where
                        #(#input_names : #input_traits,)*
                    {
                        type Input<'jvm> = ();
                        type Output<'jvm> = Local<'jvm, #ty>;

                        fn execute_with<'jvm>(
                            self,
                            jvm: &mut Jvm<'jvm>,
                            (): (),
                        ) -> duchess::Result<Self::Output<'jvm>> {
                            #(#prepare_inputs)*

                            let class = cached_class(jvm)?;

                            let env = jvm.to_env();

                            let o = env.new_object(
                                class,
                                #descriptor,
                                &[
                                    #(JValue::from(#input_names),)*
                                ]
                            )?;

                            Ok(unsafe {
                                Local::from_jni(AutoLocal::new(o, &env))
                            })
                        }
                    }

                    Impl {
                        #(#input_names: #input_names,)*
                    }
                }
            }
        );

        // useful for debugging
        // eprintln!("{output}");

        Ok(output)
    }

    fn method(&self, method: &Method) -> Result<MethodOutput, UnsupportedWildcard> {
        let mut sig = Signature::new(self.span, self.info.generics.iter().chain(&method.generics));

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
        let output_trait = sig.output_trait(&method.return_ty)?;

        let generics = self.generic_names();

        let descriptor = Literal::string(&method.descriptor.string);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &method.argument_tys);

        let method_str = Literal::string(&method.name);

        let rust_method_name = Ident::new(&method.name.to_snake_case(), self.span);
        let rust_method_type_name = Ident::new(&method.name.to_camel_case(), self.span);

        // For each method `m` in the Java type, we create a struct (named `m`)
        // that will implement the `JvmOp`.
        let struct_output = quote_spanned!(self.span =>
            #[derive(Clone)]
            #[allow(non_camel_case_types)]
            pub struct #rust_method_type_name<This, #(#input_names),*> {
                this: This,
                #(#input_names : #input_names,)*
            }
        );

        // The method signature for the extension trait.
        let trait_method = quote_spanned!(self.span =>
            type #rust_method_type_name<#(#input_names),*>: #output_trait
            where
                #(#input_names: #input_traits,)*;

            fn #rust_method_name<#(#input_names),*>(
                self,
                #(#input_names: #input_names),*
            ) -> Self::#rust_method_type_name<#(#input_names),*>
            where
                #(#input_names: #input_traits,)*;
        );

        // The method signature for the extension trait.
        let trait_impl_method = quote_spanned!(self.span =>
            type #rust_method_type_name<#(#input_names),*> = #rust_method_type_name<Self, #(#input_names,)*>
            where
                #(#input_names: #input_traits,)*;

            fn #rust_method_name<#(#input_names),*>(
                self,
                #(#input_names: #input_names),*
            ) -> #rust_method_type_name<Self, #(#input_names),*>
            where
                #(#input_names: #input_traits,)*
            {
                #rust_method_type_name {
                    this: self,
                    #(#input_names: #input_names,)*
                }
            }
        );

        // Implementation of `JvmOp` for `m` -- when executed, call the method
        // via JNI, after converting its arguments appropriately.
        let impl_output = quote_spanned!(self.span =>
            #[allow(non_camel_case_types)]
            impl<This, #(#input_names),*> JvmOp for #rust_method_type_name<This, #(#input_names),*>
            where
                This: JvmOp,
                for<'jvm> This::Output<'jvm>: AsRef<#this_ty>,
                #(#input_names: #input_traits,)*
            {
                type Input<'jvm> = This::Input<'jvm>;
                type Output<'jvm> = #output_ty;

                fn execute_with<'jvm>(
                    self,
                    jvm: &mut Jvm<'jvm>,
                    input: This::Input<'jvm>,
                ) -> duchess::Result<Self::Output<'jvm>> {
                    let this = self.this.execute_with(jvm, input)?;
                    let this: & #this_ty = this.as_ref();
                    let this = this.as_jobject();

                    #(#prepare_inputs)*

                    let env = jvm.to_env();
                    let result = env.call_method(this, #method_str, #descriptor, &[
                        #(JValue::from(#input_names),)*
                    ])?;

                    Ok(FromJValue::from_jvalue(jvm, result))
                }
            }
        );

        // useful for debugging
        // eprintln!("{trait_method}");
        // eprintln!("{trait_impl_method}");

        Ok(MethodOutput {
            method_struct: struct_output,
            trait_method,
            trait_impl_method,
            jvm_op_impl: impl_output,
        })
    }

    fn struct_name(&self) -> Ident {
        let tail = self.info.name.split('.').last().unwrap();
        Ident::new(&tail, self.span)
    }

    fn ext_trait_name(&self) -> Ident {
        let tail = self.info.name.split('.').last().unwrap();
        Ident::new(&format!("{tail}Ext"), self.span)
    }

    fn generic_names(&self) -> Vec<Ident> {
        self.info
            .generics
            .iter()
            .map(|g| java_type_parameter_ident(self.span, g))
            .collect()
    }

    fn this_type(&self) -> TokenStream {
        let s = self.struct_name();
        if self.info.generics.is_empty() {
            quote_spanned!(self.span => #s)
        } else {
            let g: Vec<Ident> = self.generic_names();
            quote_spanned!(self.span => #s < #(#g),* >)
        }
    }

    /// Returns a class name with `/`, like `java/lang/Object`.
    fn jni_class_name(&self) -> Literal {
        self.info.name.to_jni_name(self.span)
    }

    fn prepare_inputs(&self, input_names: &[Ident], input_types: &[Type]) -> Vec<TokenStream> {
        input_names
            .iter()
            .zip(input_types)
            .map(|(input_name, input_ty)| match input_ty {
                Type::Scalar(_) => quote_spanned!(self.span =>
                    let #input_name = self.#input_name.execute(jvm)?;
                ),
                Type::Ref(_) => quote_spanned!(self.span =>
                    let #input_name = self.#input_name.into_java(jvm)?;
                    let #input_name = #input_name.as_ref();
                    let #input_name = &#input_name.as_jobject();
                ),
            })
            .collect()
    }
}

struct Signature {
    span: Span,
    generics: Vec<Ident>,
    where_bounds: Vec<TokenStream>,
    capture_generics: bool,
}

/// We translate Java wildcards to Rust generics, but there are
/// limits to what we can do with this technique. For example,
/// an input `ArrayList<ArrayList<?>>` has no Rust equivalent,
/// it would be something like `ArrayList<exists<T> ArrayList<T>>`,
/// if we had `exists`. Similarly returning a type like
/// `-> ArrayList<? extends Foo>` isn't possible today, though we
/// could conceivably handle it via some kind of fresh struct or
/// `impl Trait` return value.
///
/// When we encounter cases like this, we return
/// `Err(UnsupportedWildcard)`.
struct UnsupportedWildcard;

impl Signature {
    pub fn new<'i>(span: Span, generics: impl IntoIterator<Item = &'i Id>) -> Self {
        let mut this = Signature {
            span,
            generics: vec![],
            where_bounds: vec![],
            capture_generics: true,
        };
        for generic in generics {
            let ident = this.java_type_parameter_ident(generic);
            this.generics.push(ident);
        }
        this
    }

    /// Set the `capture_generics` field to false while `op` executes,
    /// then restore its value.
    fn forbid_capture<R>(&mut self, op: impl FnOnce(&mut Self) -> R) -> R {
        let v = std::mem::replace(&mut self.capture_generics, false);
        let r = op(self);
        self.capture_generics = v;
        r
    }

    /// Generates a fresh generic type and adds it to `self.generics`.
    ///
    /// Used to manage Java wildcards. A type like `ArrayList<?>` gets
    /// translated to a Rust type like `ArrayList<Pi>` for some fresh `Pi`.
    ///
    /// See also `Self::push_where_bound`.
    fn fresh_generic(&mut self) -> Result<Ident, UnsupportedWildcard> {
        if !self.capture_generics {
            Err(UnsupportedWildcard)
        } else {
            let mut i = self.generics.len();
            let ident = Ident::new(&format!("P{}", i), self.span);
            self.generics.push(ident.clone());
            Ok(ident)
        }
    }

    /// Push a where bound into the list of where clauses that will be
    /// emitted later. Used to manage Java wildcards. A type like
    /// `ArrayList<? extends Foo>` becomes `ArrayList<X>` with a bound
    /// `X: Upcast<Foo>`.
    ///
    /// See also `Self::fresh_generic`.
    fn push_where_bound(&mut self, t: TokenStream) {
        self.where_bounds.push(t);
    }

    /// Returnss an appropriate `impl type` for a funtion that
    /// takes `ty` as input. Assumes objects are nullable.
    fn input_trait(&mut self, ty: &Type) -> Result<TokenStream, UnsupportedWildcard> {
        match ty {
            Type::Ref(ty) => {
                let t = self.java_ref_ty(ty)?;
                Ok(quote_spanned!(self.span => IntoJava<#t>))
            }
            Type::Scalar(ty) => {
                let t = self.java_scalar_ty(ty);
                Ok(quote_spanned!(self.span => IntoScalar<#t>))
            }
        }
    }

    /// Returnss an appropriate `impl type` for a funtion that
    /// returns `ty`. Assumes objects are nullable.
    fn output_type(&mut self, ty: &Option<Type>) -> Result<TokenStream, UnsupportedWildcard> {
        self.forbid_capture(|this| match ty {
            Some(Type::Ref(ty)) => {
                let t = this.java_ref_ty(ty)?;
                Ok(quote_spanned!(this.span => Option<Local<'jvm, #t>>))
            }
            Some(Type::Scalar(ty)) => {
                let t = this.java_scalar_ty(ty);
                Ok(quote_spanned!(this.span => #t))
            }
            None => Ok(quote_spanned!(this.span => ())),
        })
    }

    /// Returns an appropriate `impl type` for a funtion that
    /// returns `ty`. Assumes objects are nullable.
    fn output_trait(&mut self, ty: &Option<Type>) -> Result<TokenStream, UnsupportedWildcard> {
        self.forbid_capture(|this| match ty {
            Some(Type::Ref(ty)) => {
                let t = this.java_ref_ty(ty)?;
                Ok(quote_spanned!(this.span => IntoOptLocal<#t>))
            }
            Some(Type::Scalar(ty)) => {
                let t = this.java_scalar_ty(ty);
                Ok(quote_spanned!(this.span => IntoScalar<#t>))
            }
            None => Ok(quote_spanned!(this.span => IntoVoid)),
        })
    }

    /// For a Java type
    fn java_type(&mut self, ty: &Type) -> Result<TokenStream, UnsupportedWildcard> {
        match ty {
            Type::Ref(ty) => self.java_ref_ty(ty),

            Type::Scalar(ty) => Ok(self.java_scalar_ty(ty)),
        }
    }

    fn java_ref_ty(&mut self, ty: &RefType) -> Result<TokenStream, UnsupportedWildcard> {
        match ty {
            RefType::Class(ty) => Ok(self.class_ref_ty(ty)?),
            RefType::Array(e) => {
                let e = self.java_ref_ty(e)?;
                Ok(quote_spanned!(self.span => java::JavaArray<#e>))
            }
            RefType::TypeParameter(t) => {
                let ident = self.java_type_parameter_ident(t);
                assert!(self.generics.contains(&ident));
                Ok(quote_spanned!(self.span => #ident))
            }
            RefType::Extends(ty) => {
                let g = self.fresh_generic()?;
                let e = self.java_ref_ty(ty)?;
                self.push_where_bound(quote_spanned!(self.span => #g : AsRef<#e>));
                Ok(quote_spanned!(self.span => #g))
            }
            RefType::Super(_) => {
                let g = self.fresh_generic()?;
                // FIXME: missing where bound, really
                Ok(quote_spanned!(self.span => #g))
            }
            RefType::Wildcard => {
                let g = self.fresh_generic()?;
                Ok(quote_spanned!(self.span => #g))
            }
        }
    }

    fn class_ref_ty(&mut self, ty: &ClassRef) -> Result<TokenStream, UnsupportedWildcard> {
        let ClassRef { name, generics } = ty;
        let rust_name = name.to_module_name(self.span);
        if generics.len() == 0 {
            Ok(quote_spanned!(self.span => #rust_name))
        } else {
            let rust_tys: Vec<_> = generics
                .iter()
                .map(|t| self.java_ref_ty(t))
                .collect::<Result<_, _>>()?;
            Ok(quote_spanned!(self.span => #rust_name < #(#rust_tys),* >))
        }
    }

    fn java_type_parameter_ident(&self, t: &Id) -> Ident {
        java_type_parameter_ident(self.span, t)
    }

    fn java_scalar_ty(&self, ty: &ScalarType) -> TokenStream {
        match ty {
            ScalarType::Int => quote_spanned!(self.span => i32),
            ScalarType::Long => quote_spanned!(self.span => i64),
            ScalarType::Short => quote_spanned!(self.span => i16),
            ScalarType::Byte => quote_spanned!(self.span => i8),
            ScalarType::F64 => quote_spanned!(self.span => f64),
            ScalarType::F32 => quote_spanned!(self.span => f32),
            ScalarType::Boolean => quote_spanned!(self.span => bool),
        }
    }
}

fn java_type_parameter_ident(span: Span, t: &Id) -> Ident {
    Ident::new(&format!("J{}", t), span)
}

trait IdExt {
    fn to_jni_name(&self, span: Span) -> Literal;
    fn to_module_name(&self, span: Span) -> TokenStream;
}

impl IdExt for Id {
    fn to_jni_name(&self, _span: Span) -> Literal {
        let s = self.replace('.', "/");
        Literal::string(&s)
    }

    fn to_module_name(&self, span: Span) -> TokenStream {
        let rust_name: Vec<&str> = self.split('.').collect();
        let (struct_name, package_names) = rust_name.split_last().unwrap();
        let struct_ident = Ident::new(struct_name, span);
        let package_idents: Vec<Ident> =
            package_names.iter().map(|n| Ident::new(n, span)).collect();
        quote_spanned!(span => #(#package_idents ::)* #struct_ident)
    }
}
