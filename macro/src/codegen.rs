use crate::{
    class_info::{
        ClassRef, Constructor, Id, Method, NonRepeatingType, RefType, ScalarType, SpannedClassInfo,
        SpannedPackageInfo, Type, RootMap,
    },
    span_error::SpanError, argument::DuchessDeclaration,
};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote_spanned;
use rust_format::Formatter;

impl DuchessDeclaration {
    pub fn into_tokens(self) -> Result<TokenStream, SpanError> {
        let root_map = self.to_root_map()?;
        let () = root_map.check()?;
        root_map.into_tokens()
    }
}

impl RootMap {
    fn into_tokens(self) -> Result<TokenStream, SpanError> {
        self.into_packages().map(|p| p.into_tokens(0)).collect()
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
    fn into_tokens(self, depth: usize) -> Result<TokenStream, SpanError> {
        let name = Ident::new(&self.name, self.span);
        let inner: TokenStream = self
            .subpackages
            .into_values()
            .map(|p| p.into_tokens(depth + 1))
            .chain(self.classes.into_iter().map(|c| c.into_tokens()))
            .collect::<Result<_, _>>()?;

        let path: TokenStream = (1..depth)
            .map(|_| quote_spanned!(self.span => "::super"))
            .collect();

        Ok(quote_spanned!(self.span =>
            #[allow(unused_imports)]
            pub mod #name {
                // Import the contents of the parent module that we are created inside
                use super #path :: *;

                // Import the java package provided by duchess
                use duchess::java;

                #inner
            }
        ))
    }
}

impl SpannedClassInfo {
    pub fn into_tokens(self) -> Result<TokenStream, SpanError> {
        let struct_name = self.struct_name();
        let ext_trait_name = self.ext_trait_name();
        let cached_class = self.cached_class();
        let this_ty = self.this_type();
        let java_class_generics = self.class_generic_names();

        // Convert constructors
        let constructors: Vec<_> = self.selected_constructors()
            .map(|c| self.constructor(c))
            .collect::<Result<_, _>>()?;

        // Convert class methods (not static methods, those are different)
        let object_methods: Vec<_> = self.selected_methods()
            .filter(|m| !m.flags.is_static)
            .map(|m| self.method(m))
            .collect::<Result<_, _>>()?;

        let method_structs: Vec<_> = object_methods.iter().map(|m| &m.method_struct).collect();
        let trait_methods: Vec<_> = object_methods.iter().map(|m| &m.trait_method).collect();
        let trait_impl_methods: Vec<_> = object_methods
            .iter()
            .map(|m| &m.trait_impl_method)
            .collect();
        let jvm_op_impls: Vec<_> = object_methods.iter().map(|m| &m.jvm_op_impl).collect();
        let upcast_impls = self.upcast_impls();

        let output = quote_spanned! {
            self.span =>

            #[allow(non_camel_case_types)]
            pub struct #struct_name<#(#java_class_generics,)*> {
                _dummy: std::marker::PhantomData<(#(#java_class_generics,)*)>
            }

            #[allow(non_camel_case_types)]
            pub trait #ext_trait_name<#(#java_class_generics,)*> : duchess::JvmOp
            where
                #(#java_class_generics : JavaObject,)*
            {
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
                    objects::{AutoLocal, GlobalRef, JMethodID, JValue, JValueGen},
                    signature::ReturnType,
                    sys::jvalue,
                };
                use once_cell::sync::OnceCell;

                unsafe impl<#(#java_class_generics,)*> JavaObject for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: JavaObject,)*
                {}

                unsafe impl<#(#java_class_generics,)*> plumbing::Upcast<#struct_name<#(#java_class_generics,)*>> for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: JavaObject,)*
                {}

                #upcast_impls

                #cached_class

                #(#constructors)*

                #(#method_structs)*

                #(#jvm_op_impls)*

                #[allow(non_camel_case_types)]
                impl<This, #(#java_class_generics,)*> #ext_trait_name<#(#java_class_generics,)*> for This
                where
                    This: JvmOp,
                    for<'jvm> This::Output<'jvm>: AsRef<#this_ty>,
                    #(#java_class_generics: JavaObject,)*
                {
                    #(#trait_impl_methods)*
                }
            };
        };

        if let Ok(f) = std::env::var("DUCHESS_DEBUG") {
            if f == "*" || f == "1" || self.info.name.starts_with(&f) {
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

    fn upcast_impls(&self) -> TokenStream {
        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();
        self.info
            .extends
            .iter()
            .chain(&self.info.implements)
            .map(|r| {
                let mut sig = Signature::new(&Id::from("supertrait"), self.span, &self.info.generics);
                let tokens = sig.forbid_capture(|sig| sig.class_ref_ty(r)).unwrap();
                quote_spanned!(self.span => 
                    unsafe impl<#(#java_class_generics,)*> plumbing::Upcast<#tokens> for #struct_name<#(#java_class_generics,)*>
                    where
                        #(#java_class_generics: JavaObject,)*
                    {}
                )
            })
            .collect()
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

    fn constructor(&self, constructor: &Constructor) -> Result<TokenStream, SpanError> {
        let mut sig = Signature::new(&self.info.name, self.span, &self.info.generics);

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

        let descriptor = Literal::string(&constructor.descriptor.string);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &constructor.argument_tys);

        let output = quote_spanned!(self.span =>
            impl< #(#java_class_generics,)* > #ty
            where
                #(#java_class_generics: JavaObject,)*
            {
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
                        #(#java_class_generics: JavaObject,)*
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
                        phantom: Default::default()
                    }
                }
            }
        );

        // useful for debugging
        // eprintln!("{output}");

        Ok(output)
    }

    fn method(&self, method: &Method) -> Result<MethodOutput, SpanError> {
        let mut sig = Signature::new(
            &method.name,
            self.span,
            self.info.generics.iter().chain(&method.generics),
        );

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

        let descriptor = Literal::string(&method.descriptor.string);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &method.argument_tys);

        let method_str = Literal::string(&method.name);

        let rust_method_name = Ident::new(&method.name.to_snake_case(), self.span);
        let rust_method_type_name = Ident::new(&method.name.to_camel_case(), self.span);

        // The generic parameters declared on the Java method.
        let java_class_generics: Vec<_> = self.class_generic_names();
        let java_method_generics: Vec<_> = method
            .generics
            .iter()
            .map(|g| g.to_ident(self.span))
            .collect();

        // The generic parameters we need on the Rust method, these include:
        //
        // * a type parameter `a0` for each input
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics: Vec<_> = input_names
            .iter()
            .chain(&java_method_generics)
            .chain(sig.fresh_generics.iter())
            .collect();

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
                #(#java_method_generics: JavaObject,)*
                #(#sig_where_clauses,)*
                ;

            fn #rust_method_name<#(#rust_method_generics),*>(
                self,
                #(#input_names: #input_names),*
            ) -> Self::#rust_method_type_name<#(#rust_method_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#java_method_generics: JavaObject,)*
                #(#sig_where_clauses,)*
                ;
        );

        // The method signature for the extension trait.
        let trait_impl_method = quote_spanned!(self.span =>
            type #rust_method_type_name<#(#rust_method_generics),*> =
                #rust_method_type_name<Self, #(#method_struct_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#java_method_generics: JavaObject,)*
                #(#sig_where_clauses,)*
                ;

            fn #rust_method_name<#(#rust_method_generics),*>(
                self,
                #(#input_names: #input_names),*
            ) -> Self::#rust_method_type_name<#(#rust_method_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#java_method_generics: JavaObject,)*
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
                for<'jvm> This::Output<'jvm>: AsRef<#this_ty>,
                #(#input_names: #input_traits,)*
                #(#java_class_generics: JavaObject,)*
                #(#java_method_generics: JavaObject,)*
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
            method_struct,
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

    fn class_generic_names(&self) -> Vec<Ident> {
        self.info
            .generics
            .iter()
            .map(|g| g.to_ident(self.span))
            .collect()
    }

    fn this_type(&self) -> TokenStream {
        let s = self.struct_name();
        if self.info.generics.is_empty() {
            quote_spanned!(self.span => #s)
        } else {
            let g: Vec<Ident> = self.class_generic_names();
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
            .map(|(input_name, input_ty)| match input_ty.to_non_repeating() {
                NonRepeatingType::Scalar(_) => quote_spanned!(self.span =>
                    let #input_name = self.#input_name.execute(jvm)?;
                ),
                NonRepeatingType::Ref(_) => quote_spanned!(self.span =>
                    let #input_name = self.#input_name.into_java(jvm)?;
                    let #input_name = #input_name.as_ref();
                    let #input_name = &#input_name.as_jobject();
                ),
            })
            .collect()
    }
}

struct Signature {
    method_name: Id,
    span: Span,
    in_scope_generics: Vec<Id>,
    fresh_generics: Vec<Ident>,
    where_clauses: Vec<TokenStream>,
    capture_generics: bool,
}

impl Signature {
    pub fn new<'i>(
        method_name: &Id,
        span: Span,
        in_scope_generics: impl IntoIterator<Item = &'i Id>,
    ) -> Self {
        Signature {
            method_name: method_name.clone(),
            span,
            in_scope_generics: in_scope_generics.into_iter().cloned().collect(),
            fresh_generics: vec![],
            where_clauses: vec![],
            capture_generics: true,
        }
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
    fn fresh_generic(&mut self) -> Result<Ident, SpanError> {
        if !self.capture_generics {
            Err(SpanError {
                span: self.span,
                message: format!("unsupported wildcards in `{}`", self.method_name),
            })
        } else {
            let mut i = self.fresh_generics.len();
            loop {
                let ident = Ident::new(&format!("Capture{}", i), self.span);
                if !self.fresh_generics.contains(&ident) {
                    self.fresh_generics.push(ident.clone());
                    self.where_clauses
                        .push(quote_spanned!(self.span => #ident : JavaObject));
                    return Ok(ident);
                }
                i += 1;
            }
        }
    }

    /// Push a where bound into the list of where clauses that will be
    /// emitted later. Used to manage Java wildcards. A type like
    /// `ArrayList<? extends Foo>` becomes `ArrayList<X>` with a bound
    /// `X: Upcast<Foo>`.
    ///
    /// See also `Self::fresh_generic`.
    fn push_where_bound(&mut self, t: TokenStream) {
        self.where_clauses.push(t);
    }

    /// Returns an appropriate `impl type` for a funtion that
    /// takes `ty` as input. Assumes objects are nullable.
    fn input_trait(&mut self, ty: &Type) -> Result<TokenStream, SpanError> {
        match ty.to_non_repeating() {
            NonRepeatingType::Ref(ty) => {
                let t = self.java_ref_ty(&ty)?;
                Ok(quote_spanned!(self.span => duchess::IntoJava<#t>))
            }
            NonRepeatingType::Scalar(ty) => {
                let t = self.java_scalar_ty(&ty);
                Ok(quote_spanned!(self.span => duchess::IntoScalar<#t>))
            }
        }
    }

    /// Returns an appropriate `impl type` for a funtion that
    /// returns `ty`. Assumes objects are nullable.
    fn output_type(&mut self, ty: &Option<Type>) -> Result<TokenStream, SpanError> {
        self.forbid_capture(|this| match ty.as_ref().map(|ty| ty.to_non_repeating()) {
            Some(NonRepeatingType::Ref(ty)) => {
                let t = this.java_ref_ty(&ty)?;
                Ok(quote_spanned!(this.span => Option<Local<'jvm, #t>>))
            }
            Some(NonRepeatingType::Scalar(ty)) => {
                let t = this.java_scalar_ty(&ty);
                Ok(quote_spanned!(this.span => #t))
            }
            None => Ok(quote_spanned!(this.span => ())),
        })
    }

    /// Returns an appropriate trait for a method that
    /// returns `ty`. Assumes objects are nullable.
    fn method_trait(&mut self, ty: &Option<Type>) -> Result<TokenStream, SpanError> {
        self.forbid_capture(|this| match ty.as_ref().map(|ty| ty.to_non_repeating()) {
            Some(NonRepeatingType::Ref(ty)) => {
                let t = this.java_ref_ty(&ty)?;
                Ok(quote_spanned!(this.span => duchess::JavaMethod<Self, #t>))
            }
            Some(NonRepeatingType::Scalar(ty)) => {
                let t = this.java_scalar_ty(&ty);
                Ok(quote_spanned!(this.span => duchess::ScalarMethod<Self, #t>))
            }
            None => Ok(quote_spanned!(this.span => duchess::VoidMethod<Self>)),
        })
    }

    fn java_ty(&mut self, ty: &Type) -> Result<TokenStream, SpanError> {
        match &ty.to_non_repeating() {
            NonRepeatingType::Ref(ty) => self.java_ref_ty(ty),
            NonRepeatingType::Scalar(ty) => Ok(self.java_scalar_ty(ty)),
        }
    }

    fn java_ref_ty(&mut self, ty: &RefType) -> Result<TokenStream, SpanError> {
        match ty {
            RefType::Class(ty) => Ok(self.class_ref_ty(ty)?),
            RefType::Array(e) => {
                let e = self.java_ty(e)?;
                Ok(quote_spanned!(self.span => java::Array<#e>))
            }
            RefType::TypeParameter(t) => {
                assert!(
                    self.in_scope_generics.contains(&t),
                    "generic type parameter `{:?}` not among in-scope parameters: {:?}",
                    t, 
                    self.in_scope_generics,
                );
                let t = t.to_ident(self.span);
                Ok(quote_spanned!(self.span => #t))
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

    fn class_ref_ty(&mut self, ty: &ClassRef) -> Result<TokenStream, SpanError> {
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

    fn java_scalar_ty(&self, ty: &ScalarType) -> TokenStream {
        match ty {
            ScalarType::Char => quote_spanned!(self.span => u16),
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
