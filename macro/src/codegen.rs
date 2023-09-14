use crate::{
    argument::DuchessDeclaration,
    class_info::{
        ClassInfo, Constructor, DotId, Field, Id, Method, NonRepeatingType, RootMap,
        SpannedPackageInfo, Type,
    },
    reflect::Reflector,
    signature::Signature,
    upcasts::Upcasts,
};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote_spanned;

impl DuchessDeclaration {
    pub fn to_tokens(&self) -> syn::Result<TokenStream> {
        let reflector = &mut Reflector::default();
        let root_map = self.to_root_map(reflector)?;
        let () = root_map.check(reflector)?;
        root_map.to_tokens(reflector)
    }
}

impl RootMap {
    fn to_tokens(self, reflector: &mut Reflector) -> syn::Result<TokenStream> {
        self.to_packages()
            .map(|p| p.to_tokens(&[], &self, reflector))
            .collect()
    }
}

impl SpannedPackageInfo {
    fn to_tokens(
        &self,
        parents: &[Id],
        root_map: &RootMap,
        reflector: &mut Reflector,
    ) -> syn::Result<TokenStream> {
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
            .map(|class_id| root_map.classes[class_id].to_tokens(&root_map.upcasts))
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
    pub fn to_tokens(&self, upcasts: &Upcasts) -> syn::Result<TokenStream> {
        let struct_name = self.struct_name();
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
            .filter(|m| self.should_mirror_in_rust(m.flags.privacy))
            .filter(|m| m.flags.is_static)
            .map(|m| self.static_method(m))
            .collect::<Result<_, _>>()?;

        // Convert instance methods (not static methods, those are different)
        let op_methods: Vec<_> = self
            .methods
            .iter()
            .filter(|m| self.should_mirror_in_rust(m.flags.privacy))
            .filter(|m| !m.flags.is_static)
            .map(|m| self.op_struct_method(m))
            .collect::<Result<_, _>>()?;

        // Convert instance methods (not static methods, those are different)
        let obj_methods: Vec<_> = self
            .methods
            .iter()
            .filter(|m| self.should_mirror_in_rust(m.flags.privacy))
            .filter(|m| !m.flags.is_static)
            .map(|m| self.obj_struct_method(m))
            .collect::<Result<_, _>>()?;

        let assoc_struct_declarations = self.assoc_structs(upcasts, op_methods, obj_methods)?;

        // Convert instance methods of the form `Foo::method`
        let inherent_object_methods: Vec<_> = self
            .methods
            .iter()
            .filter(|m| self.should_mirror_in_rust(m.flags.privacy))
            .filter(|m| !m.flags.is_static)
            .map(|m| self.inherent_object_method(m))
            .collect::<Result<_, _>>()?;

        // Generate static field getters
        let static_field_getters: Vec<_> = self
            .fields
            .iter()
            .filter(|f: &&Field| self.should_mirror_in_rust(f.flags.privacy))
            .filter(|f| f.flags.is_static)
            .map(|f| self.static_field_getter(f))
            .collect::<Result<_, _>>()?;

        let upcast_impls = self.upcast_impls(upcasts)?;

        let output = quote_spanned! {
            self.span =>

            #[allow(non_camel_case_types)]
            pub struct #struct_name<#(#java_class_generics_with_defaults,)*> {
                _dummy: ::core::marker::PhantomData<(#(#java_class_generics,)*)>
            }

            // Hide other generated items
            #[allow(unused_imports)]
            #[allow(nonstandard_style)]
            const _: () = {
                #assoc_struct_declarations

                unsafe impl<#(#java_class_generics,)*> duchess::JavaObject for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    #cached_class
                }

                impl<#(#java_class_generics,)*> ::core::convert::AsRef<#struct_name<#(#java_class_generics,)*>> for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    fn as_ref(&self) -> &#struct_name<#(#java_class_generics,)*> {
                        self
                    }
                }

                impl<#(#java_class_generics,)*> ::core::ops::Deref for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    type Target = <Self as duchess::plumbing::JavaView>::OfObj<Self>;

                    fn deref(&self) -> &Self::Target {
                        duchess::plumbing::FromRef::from_ref(self)
                    }
                }

                impl<#(#java_class_generics,)*> duchess::prelude::JDeref for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    fn jderef(&self) -> &Self {
                        self
                    }
                }

                impl<#(#java_class_generics,)*> duchess::prelude::TryJDeref for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    type Java = Self;

                    fn try_jderef(&self) -> duchess::Nullable<&Self> {
                        Ok(self)
                    }
                }

                // Reflexive upcast impl
                unsafe impl<#(#java_class_generics,)*> duchess::plumbing::Upcast<#struct_name<#(#java_class_generics,)*>> for #struct_name<#(#java_class_generics,)*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {}

                // Other upcast impls
                #upcast_impls

                impl< #(#java_class_generics,)* > #this_ty
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                {
                    #(#constructors)*

                    #(#static_methods)*

                    #(#static_field_getters)*

                    #(#inherent_object_methods)*
                }
            };
        };

        crate::debug_tokens(&self.name, &output);

        Ok(output)
    }

    /// Construct the various declarations related to the op struct,
    /// with the exception of any methods that must be invoked.
    fn assoc_structs(
        &self,
        upcasts: &Upcasts,
        op_struct_methods: Vec<TokenStream>,
        obj_struct_methods: Vec<TokenStream>,
    ) -> syn::Result<TokenStream> {
        let name = self.struct_name();
        let java_class_generics = self.class_generic_names();

        // Names for the `J` and `N` parameters.
        // There is no particular reason to create `Ident` for these
        // except that I decided to pretend we had hygienic procedural
        // macros in this particular piece of code for some reason.
        let j = Ident::new("J", self.span);
        let n = Ident::new("N", self.span);

        // Subtle: the struct itself does not require
        // that `J: IntoJava<Self>` or `N: FromRef<J>`.
        // Those requirements are placed only on the `Deref` impls.

        let struct_definition = |struct_name: &Ident| {
            quote_spanned!(self.span =>
                #[repr(transparent)]
                pub struct #struct_name<#(#java_class_generics,)* #j, #n> {
                    this: #j,
                    phantom: ::core::marker::PhantomData<(#name<#(#java_class_generics),*>, #n)>,
                }
            )
        };

        let deref_impl = |struct_name: &Ident| {
            quote_spanned!(self.span =>
                impl<#(#java_class_generics,)* #j, #n> ::core::ops::Deref
                for #struct_name<#(#java_class_generics,)* #j, #n>
                where
                    #n: duchess::plumbing::FromRef<#j>,
                {
                    type Target = #n;

                    fn deref(&self) -> &#n {
                        duchess::plumbing::FromRef::from_ref(&self.this)
                    }
                }
            )
        };

        let from_ref_impl = |struct_name: &Ident| {
            quote_spanned!(self.span =>
                impl<#(#java_class_generics,)* #j, #n> duchess::plumbing::FromRef<#j> for #struct_name<#(#java_class_generics,)* #j, #n> {
                    fn from_ref(j: &J) -> &Self {
                        // This is safe because of the `#[repr(transparent)]`
                        // on the struct declaration.
                        unsafe {
                            ::core::mem::transmute::<&J, &Self>(j)
                        }
                    }
                }
            )
        };

        // Construct the default value for the "next" (#n) parameter.
        let mro = self.mro(upcasts)?;

        let op_name = Id::from(format!("ViewAs{}Op", self.name.class_name())).to_ident(self.span);
        let op_mro_tokens = self.mro_tokens(&j, "OfOpWith", &mro);

        let obj_name = Id::from(format!("ViewAs{}Obj", self.name.class_name())).to_ident(self.span);
        let obj_mro_tokens = self.mro_tokens(&j, "OfObjWith", &mro);

        let all_names = &[&op_name, &obj_name];

        let this_ty = self.this_type();

        let other_impls = quote_spanned!(self.span =>
            impl<#(#java_class_generics,)*> duchess::plumbing::JavaView for #name<#(#java_class_generics,)*>
            {
                type OfOp<#j> = #op_name<#(#java_class_generics,)* #j, #op_mro_tokens>;

                type OfOpWith<#j, #n> = #op_name<#(#java_class_generics,)* #j, #n>
                where
                    N: duchess::plumbing::FromRef<J>;

                type OfObj<#j> = #obj_name<#(#java_class_generics,)* #j, #obj_mro_tokens>;

                type OfObjWith<#j, #n> = #obj_name<#(#java_class_generics,)* #j, #n>
                where
                    N: duchess::plumbing::FromRef<J>;
            }

            impl<#(#java_class_generics,)* #j, #n> #op_name<#(#java_class_generics,)* #j, #n>
            where
                #(#java_class_generics: duchess::JavaObject,)*
                #j: duchess::prelude::IntoJava<#name<#(#java_class_generics,)*>>,
                #n: duchess::plumbing::FromRef<#j>,
            {
                #(#op_struct_methods)*
            }

            impl<#(#java_class_generics,)* #j, #n> #obj_name<#(#java_class_generics,)* #j, #n>
            where
                #(#java_class_generics: duchess::JavaObject,)*
                for<'jvm> &'jvm #j: duchess::prelude::IntoJava<#this_ty>,
            {
                #(#obj_struct_methods)*
            }
        );

        let declarations: TokenStream = all_names
            .iter()
            .copied()
            .flat_map(|n| vec![struct_definition(n), deref_impl(n), from_ref_impl(n)])
            .chain(Some(other_impls))
            .collect();

        Ok(declarations)
    }

    /// Constructs the default "next" type for our [op struct].
    /// This is based on the method resolution order (mro) for the
    /// current type. For example, if `Foo` extends `Bar`, then the
    /// result for `Foo` would be
    /// `Bar::OfOpWith<J, java::lang::Object::OfOpWith<J, ()>>`.
    ///
    /// [op struct]: https://duchess-rs.github.io/duchess/methods.html#op-structs
    fn mro_tokens(&self, j: &Ident, assoc_name: &str, mro: &[TokenStream]) -> TokenStream {
        let Some((head, tail)) = mro.split_first() else {
            return quote_spanned!(self.span => ());
        };

        let tail_tokens = self.mro_tokens(j, assoc_name, tail);

        let assoc_ident = Ident::new(assoc_name, self.span);
        quote_spanned!(self.span =>
            <#head as duchess::plumbing::JavaView>::#assoc_ident<#j, #tail_tokens>
        )
    }

    /// Returns the ["method resolution order"][mro] for self. This is a series of
    /// supertypes (classes or interfaces) ordered such that the more specific types
    /// appear first. The returned list only includes "proper" supertypes, it does not
    /// include the current class.
    ///
    /// FIXME: The returned list contains the right items, but is in an arbitary order,
    /// and is not following the documented order. The result is that calls may wind up
    /// calling methods from supertypes instead of subtypes. This only matters if subtypes
    /// refine the return type.
    ///
    /// [mro]: https://duchess-rs.github.io/duchess/methods.html#method-resolution-order
    fn mro(&self, upcasts: &Upcasts) -> syn::Result<Vec<TokenStream>> {
        let class_refs = upcasts.upcasts_for_generated_class(&self.name);
        class_refs
            .iter()
            .map(|r| {
                let mut sig = Signature::new(&Id::from("supertrait"), self.span, &[])
                    .with_internal_generics(&self.generics)?;
                Ok(sig.forbid_capture(|sig| sig.class_ref_ty(r)).unwrap())
            })
            .collect()
    }

    fn upcast_impls(&self, upcasts: &Upcasts) -> syn::Result<TokenStream> {
        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();
        Ok(self.mro(upcasts)?
            .into_iter()
            .map(|tokens| {
                quote_spanned!(self.span =>
                    unsafe impl<#(#java_class_generics,)*> duchess::plumbing::Upcast<#tokens> for #struct_name<#(#java_class_generics,)*>
                    where
                        #(#java_class_generics: duchess::JavaObject,)*
                    {}
                )
            })
            .collect())
    }

    fn cached_class(&self) -> TokenStream {
        let jni_class_name = self.jni_class_name();

        quote_spanned! {
            self.span =>
            fn class<'jvm>(jvm: &mut duchess::Jvm<'jvm>) -> duchess::Result<'jvm, duchess::Local<'jvm, java::lang::Class>> {
                static CLASS: duchess::plumbing::once_cell::sync::OnceCell<duchess::Global<java::lang::Class>> = duchess::plumbing::once_cell::sync::OnceCell::new();
                let global = CLASS.get_or_try_init::<_, duchess::Error<duchess::Local<java::lang::Throwable>>>(|| {
                    let class = duchess::plumbing::find_class(jvm, #jni_class_name)?;
                    Ok(jvm.global(&class))
                })?;
                Ok(jvm.local(global))
            }
        }
    }

    fn constructor(&self, constructor: &Constructor) -> syn::Result<TokenStream> {
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
        let output_trait = quote_spanned!(self.span => duchess::prelude::JavaConstructor<#ty>);

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
                struct Impl<
                    #(#java_class_generics,)*
                    #(#input_names),*
                > {
                    #(#input_names: #input_names,)*
                    phantom: ::core::marker::PhantomData<(
                        #(#java_class_generics,)*
                    )>,
                }

                impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                > ::core::marker::Copy for Impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                >
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                    #(#input_names : #input_traits,)*
                {
                }

                impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                > ::core::clone::Clone for Impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                >
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                    #(#input_names : #input_traits,)*
                {
                    fn clone(&self) -> Self {
                        *self
                    }
                }

                impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                > duchess::prelude::JvmOp for Impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                >
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                    #(#input_names : #input_traits,)*
                {
                    type Output<'jvm> = duchess::Local<'jvm, #ty>;

                    fn execute_with<'jvm>(
                        self,
                        jvm: &mut duchess::Jvm<'jvm>,
                    ) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                        #(#prepare_inputs)*

                        let class = <#ty as duchess::JavaObject>::class(jvm)?;

                        // Cache the method id for the constructor -- note that we only have one cache
                        // no matter how many generic monomorphizations there are. This makes sense
                        // given Java's erased-based generics system.
                        static CONSTRUCTOR: duchess::plumbing::once_cell::sync::OnceCell<duchess::plumbing::MethodPtr> = duchess::plumbing::once_cell::sync::OnceCell::new();
                        let constructor = CONSTRUCTOR.get_or_try_init(|| {
                            duchess::plumbing::find_constructor(jvm, &class, #jni_descriptor)
                        })?;

                        let env = jvm.env();
                        let obj: ::core::option::Option<duchess::Local<#ty>> = unsafe {
                            env.invoke_checked(|env| env.NewObjectA, |env, f| f(
                                env,
                                duchess::plumbing::JavaObjectExt::as_raw(&*class).as_ptr(),
                                constructor.as_ptr(),
                                [
                                    #(duchess::plumbing::IntoJniValue::into_jni_value(#input_names),)*
                                ].as_ptr(),
                            ))
                        }?;
                        obj.ok_or_else(|| {
                            // NewObjectA should only return a null pointer when an exception occurred in the
                            // constructor, so reaching here is a strange JVM state
                            duchess::Error::JvmInternal(format!(
                                "failed to create new `{}` via constructor `{}`",
                                #name, #descriptor,
                            ))
                        })
                    }
                }

                impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                > ::core::ops::Deref for Impl<
                    #(#java_class_generics,)*
                    #(#input_names,)*
                > {
                    type Target = <#ty as duchess::plumbing::JavaView>::OfOp<Self>;

                    fn deref(&self) -> &Self::Target {
                        <Self::Target as duchess::plumbing::FromRef<_>>::from_ref(self)
                    }
                }

                Impl {
                    #(#input_names: #input_names,)*
                    phantom: ::core::default::Default::default()
                }
            }
        );

        // useful for debugging
        // eprintln!("{output}");

        Ok(output)
    }

    /// Generates code for the methods that goes on the `ops` object.
    ///
    ///
    /// NB. This function (particularly the JvmOp impl) has significant overlap with `static_method`
    /// and `static_field_getter`, so if you make changes here, you may well need changes there.
    fn op_struct_method(&self, method: &Method) -> syn::Result<TokenStream> {
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

        // The "output trait" is the trait bounds we declare for the user,
        // e.g., for a method like `Foo method()`, we will declare a
        // Rust method `-> impl JavaMethod<Foo>`, and this variable
        // would be `JavaMethod<Foo>`.
        let output_trait = sig.method_trait(&method.return_ty)?;

        let rust_method_name = Id::from(method.name.to_snake_case()).to_ident(self.span);

        // The generic parameters we need on the Rust method, these include:
        //
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics = &sig.rust_generics;

        // The final where clauses we need on the method.
        // This includes the bounds declared in Java but also
        // other bounds we added as we converted input types
        // to account for captures.
        let sig_where_clauses = &sig.where_clauses;

        let this_ty = self.this_type();

        let inherent_method = quote_spanned!(self.span =>
            pub fn #rust_method_name<#(#rust_method_generics),*>(
                &self,
                #(#input_names: impl #input_traits),*
            ) -> impl #output_trait
            where
                #(#sig_where_clauses,)*
            {
                <#this_ty>::#rust_method_name(
                    self.this,
                    #(#input_names,)*
                )
            }
        );

        Ok(inherent_method)
    }

    fn obj_struct_method(&self, method: &Method) -> syn::Result<TokenStream> {
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

        // The "output trait" is the trait bounds we declare for the user,
        // e.g., for a method like `Foo method()`, we will declare a
        // Rust method `-> impl JavaMethod<Foo>`, and this variable
        // would be `JavaMethod<Foo>`.
        let output_trait = sig.method_trait(&method.return_ty)?;

        let rust_method_name = Id::from(method.name.to_snake_case()).to_ident(self.span);

        // The generic parameters we need on the Rust method, these include:
        //
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics = &sig.rust_generics;

        // The final where clauses we need on the method.
        // This includes the bounds declared in Java but also
        // other bounds we added as we converted input types
        // to account for captures.
        let sig_where_clauses = &sig.where_clauses;

        let this_ty = self.this_type();

        let inherent_method = quote_spanned!(self.span =>
            pub fn #rust_method_name<'a, #(#rust_method_generics),*>(
                &'a self,
                #(#input_names: impl #input_traits + 'a),*
            ) -> impl #output_trait + 'a
            where
                #(#sig_where_clauses,)*
            {
                <#this_ty>::#rust_method_name(
                    &self.this,
                    #(#input_names,)*
                )
            }
        );

        Ok(inherent_method)
    }

    fn inherent_object_method(&self, method: &Method) -> syn::Result<TokenStream> {
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

        // The "output type" is the actual type returned by this method,
        // e.g., `Option<Local<Foo>>`.
        let output_ty = sig.output_type(&method.return_ty)?;

        // The "output trait" is the trait bounds we declare for the user,
        // e.g., for a method like `Foo method()`, we will declare a
        // Rust method `-> impl JavaMethod<Foo>`, and this variable
        // would be `JavaMethod<Foo>`.
        let output_trait = sig.method_trait(&method.return_ty)?;

        // The appropriate JNI function to call this method.
        let jni_call_fn = sig.jni_call_fn(&method.return_ty)?;

        // If this method returns a java object, then this is the
        // Rust type representing the java class/interface that is returned
        // (e.g., `Some(java::lang::Object)`).
        let java_ref_output_ty = match &method.return_ty {
            Some(java_return_type) => {
                sig.forbid_capture(|sig| sig.java_ty_if_ref(java_return_type))?
            }
            None => None,
        };

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
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics = &sig.rust_generics;

        // The generic parameters we need on the *method struct* (which will implement the `JvmOp`).
        // These include the class generics plus all the generics from the method,
        // plus a type parameter `a0` for each input
        let this = Ident::new("this", self.span);
        let method_struct_generics: Vec<_> = java_class_generics
            .iter()
            .chain(rust_method_generics)
            .chain(Some(&this))
            .chain(&input_names)
            .collect();

        // For each method `m` in the Java type, we create a struct (named `m`)
        // that will implement the `JvmOp`.
        let method_struct = quote_spanned!(self.span =>
            pub struct #rust_method_type_name<
                #(#method_struct_generics,)*
            > {
                #this: #this,
                #(#input_names : #input_names,)*
                phantom: ::core::marker::PhantomData<(
                    #(#method_struct_generics,)*
                )>,
            }
        );

        // The final where clauses we need on the method.
        // This includes the bounds declared in Java but also
        // other bounds we added as we converted input types
        // to account for captures.
        let sig_where_clauses = &sig.where_clauses;

        // The Rust type of the class defining this method.
        let this_ty = self.this_type();

        // Implementation of `JvmOp` for `m` -- when executed, call the method
        // via JNI, after converting its arguments appropriately.
        let jvmop_impl = quote_spanned!(self.span =>
            impl<#(#method_struct_generics),*> ::core::marker::Copy
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #this: duchess::prelude::IntoJava<#this_ty>,
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {}

            impl<#(#method_struct_generics),*> ::core::clone::Clone
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #this: duchess::prelude::IntoJava<#this_ty>,
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<#(#method_struct_generics),*> duchess::prelude::JvmOp
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #this: duchess::prelude::IntoJava<#this_ty>,
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                type Output<'jvm> = #output_ty;

                fn execute_with<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                    let this = self.#this.into_java(jvm)?;
                    let this: & #this_ty = duchess::prelude::AsJRef::as_jref(&this)?;
                    let this = duchess::plumbing::JavaObjectExt::as_raw(this);

                    #(#prepare_inputs)*

                    // Cache the method id for this method -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static METHOD: duchess::plumbing::once_cell::sync::OnceCell<duchess::plumbing::MethodPtr> = duchess::plumbing::once_cell::sync::OnceCell::new();
                    let method = METHOD.get_or_try_init(|| {
                        let class = <#this_ty as duchess::JavaObject>::class(jvm)?;
                        duchess::plumbing::find_method(jvm, &class, #jni_method, #jni_descriptor, false)
                    })?;

                    unsafe {
                        jvm.env().invoke_checked(|env| env.#jni_call_fn, |env, f| f(
                            env,
                            this.as_ptr(),
                            method.as_ptr(),
                            [
                                #(duchess::plumbing::IntoJniValue::into_jni_value(#input_names),)*
                            ].as_ptr(),
                        ))
                    }
                }
            }
        );

        // If we return a Java object, then deref to its op struct.
        // See [method docs] for more details.
        // [method docs]:https://duchess-rs.github.io/duchess/methods.html
        let deref_impl = java_ref_output_ty.map(|java_ref_output_ty| {
            quote_spanned!(self.span =>
                impl<#(#method_struct_generics),*> ::core::ops::Deref
                for #rust_method_type_name<#(#method_struct_generics),*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                    #(#sig_where_clauses,)*
                {
                    type Target = <#java_ref_output_ty as duchess::plumbing::JavaView>::OfOp<Self>;

                    fn deref(&self) -> &Self::Target {
                        <Self::Target as duchess::plumbing::FromRef<_>>::from_ref(self)
                    }
                }
            )
        });

        let inherent_method = quote_spanned!(self.span =>
            pub fn #rust_method_name<#(#rust_method_generics),*>(
                #this: impl duchess::prelude::IntoJava<#this_ty>,
                #(#input_names: impl #input_traits),*
            ) -> impl #output_trait
            where
                #(#sig_where_clauses,)*
            {
                #method_struct

                #jvmop_impl

                #deref_impl

                #rust_method_type_name {
                    #this: #this,
                    #(#input_names: #input_names,)*
                    phantom: ::core::default::Default::default(),
                }
            }
        );

        Ok(inherent_method)
    }

    /// Generates a static method declaration that should be part of the inherent methods
    /// for the struct. Unlike instance methods, static methods can be totally self-contained.
    ///
    /// NB. This function (particularly the JvmOp impl) has significant overlap with `object_method`
    /// and `static_field_getter`, so if you make changes here, you may well need changes there.
    fn static_method(&self, method: &Method) -> syn::Result<TokenStream> {
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

        // If this method returns a java object, then this is the
        // Rust type representing the java class/interface that is returned
        // (e.g., `Some(java::lang::Object)`).
        let java_ref_output_ty = match &method.return_ty {
            Some(java_return_type) => {
                sig.forbid_capture(|sig| sig.java_ty_if_ref(java_return_type))?
            }
            None => None,
        };

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
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics = &sig.rust_generics;

        // The generic parameters we need on the *method struct* (which will implement the `JvmOp`).
        // These include the class generics plus all the generics from the method,
        // plus a type parameter `a0` for each input.
        let method_struct_generics: Vec<_> = java_class_generics
            .iter()
            .chain(rust_method_generics)
            .chain(&input_names)
            .collect();

        // For each method `m` in the Java type, we create a struct (named `m`)
        // that will implement the `JvmOp`.
        let method_struct = quote_spanned!(self.span =>
            pub struct #rust_method_type_name<
                #(#method_struct_generics,)*
            > {
                #(#input_names : #input_names,)*
                phantom: ::core::marker::PhantomData<(
                    #(#method_struct_generics,)*
                )>,
            }
        );

        let sig_where_clauses = &sig.where_clauses;

        // Implementation of `JvmOp` for `m` -- when executed, call the method
        // via JNI, after converting its arguments appropriately.
        let this_ty = self.this_type();
        let jvmop_impl = quote_spanned!(self.span =>
            impl<#(#method_struct_generics),*> ::core::marker::Copy
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
            }

            impl<#(#method_struct_generics),*> ::core::clone::Clone
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<#(#method_struct_generics),*> duchess::prelude::JvmOp
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #(#input_names: #input_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                type Output<'jvm> = #output_ty;

                fn execute_with<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                    #(#prepare_inputs)*

                    // Cache the method id for this method -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static METHOD: duchess::plumbing::once_cell::sync::OnceCell<duchess::plumbing::MethodPtr> = duchess::plumbing::once_cell::sync::OnceCell::new();
                    let method = METHOD.get_or_try_init(|| {
                        let class = <#this_ty as duchess::JavaObject>::class(jvm)?;
                        duchess::plumbing::find_method(jvm, &class, #jni_method, #jni_descriptor, true)
                    })?;

                    let class = <#this_ty as duchess::JavaObject>::class(jvm)?;
                    unsafe {
                        jvm.env().invoke_checked(|env| env.#jni_call_fn, |env, f| f(
                            env,
                            duchess::plumbing::JavaObjectExt::as_raw(&*class).as_ptr(),
                            method.as_ptr(),
                            [
                                #(duchess::plumbing::IntoJniValue::into_jni_value(#input_names),)*
                            ].as_ptr(),
                        ))
                    }
                }
            }
        );

        // If we return a Java object, then deref to its op struct.
        // See [method docs] for more details.
        // [method docs]:https://duchess-rs.github.io/duchess/methods.html
        let deref_impl = java_ref_output_ty.map(|java_ref_output_ty| {
            quote_spanned!(self.span =>
                impl<#(#method_struct_generics),*> ::core::ops::Deref
                for #rust_method_type_name<#(#method_struct_generics),*>
                where
                    #(#java_class_generics: duchess::JavaObject,)*
                    #(#sig_where_clauses,)*
                {
                    type Target = <#java_ref_output_ty as duchess::plumbing::JavaView>::OfOp<Self>;

                    fn deref(&self) -> &Self::Target {
                        <Self::Target as duchess::plumbing::FromRef<_>>::from_ref(self)
                    }
                }
            )
        });

        let inherent_method = quote_spanned!(self.span =>
            pub fn #rust_method_name<#(#rust_method_generics),*>(
                #(#input_names: impl #input_traits),*
            ) -> impl #output_trait
            where
                #(#sig_where_clauses,)*
            {
                #method_struct

                #jvmop_impl

                #deref_impl

                #rust_method_type_name {
                    #(#input_names: #input_names,)*
                    phantom: ::core::default::Default::default(),
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
    fn static_field_getter(&self, field: &Field) -> syn::Result<TokenStream> {
        assert!(field.flags.is_static);

        let mut sig = Signature::new(&field.name, self.span, &self.generics);

        let output_ty = sig.non_void_output_type(&field.ty)?;
        let output_trait = sig.field_trait(&field.ty)?;
        let jni_field_fn = sig.jni_static_field_get_fn(&field.ty)?;

        let jni_field = jni_c_str(&*field.name, self.span);
        let jni_descriptor = jni_c_str(&field.ty.descriptor(), self.span);

        let rust_field_name =
            Id::from(format!("get_{}", field.name.to_snake_case())).to_ident(self.span);
        let rust_field_type_name =
            Id::from(format!("{}Getter", field.name.to_camel_case())).to_ident(self.span);

        // The generic parameters declared on the Java method.
        let java_class_generics: Vec<_> = self.class_generic_names();

        // The generic parameters we need on the *method struct* (which will implement the `JvmOp`).
        // These include the class generics plus all the generics from the method.
        let field_struct_generics: Vec<_> = java_class_generics.clone(); // XX: Unnecessary clone

        // For each field `f` in the Java type, we create a struct (named `<f>Getter`)
        // that will implement the `JvmOp`.
        let field_struct = quote_spanned!(self.span =>
            pub struct #rust_field_type_name<
                #(#field_struct_generics,)*
            > {
                phantom: ::core::marker::PhantomData<(
                    #(#field_struct_generics,)*
                )>,
            }
        );

        let sig_where_clauses = &sig.where_clauses;

        // Implementation of `JvmOp` for `f` -- when executed, call the method
        // via JNI, after converting its arguments appropriately.
        let this_ty = self.this_type();
        let jvmop_impl = quote_spanned!(self.span =>
            impl<#(#field_struct_generics),*> duchess::prelude::JvmOp
            for #rust_field_type_name<#(#field_struct_generics),*>
            where
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                type Output<'jvm> = #output_ty;

                fn execute_with<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::Result<'jvm, Self::Output<'jvm>> {

                    // Cache the field id for this field -- note that we only have one cache
                    // no matter how many generic monomorphizations there are. This makes sense
                    // given Java's erased-based generics system.
                    static FIELD: duchess::plumbing::once_cell::sync::OnceCell<duchess::plumbing::FieldPtr> = duchess::plumbing::once_cell::sync::OnceCell::new();
                    let field = FIELD.get_or_try_init(|| {
                        let class = <#this_ty as duchess::JavaObject>::class(jvm)?;
                        duchess::plumbing::find_field(jvm, &class, #jni_field, #jni_descriptor, true)
                    })?;

                    let class = <#this_ty as duchess::JavaObject>::class(jvm)?;
                    unsafe {
                        jvm.env().invoke_checked(|env| env.#jni_field_fn, |env, f| f(
                            env,
                            duchess::plumbing::JavaObjectExt::as_raw(&*class).as_ptr(),
                            field.as_ptr(),
                        ))
                    }
                }
            }

            impl<#(#field_struct_generics),*> ::core::marker::Copy for #rust_field_type_name<#(#field_struct_generics),*>
            where
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
            }

            impl<#(#field_struct_generics),*> ::core::clone::Clone for #rust_field_type_name<#(#field_struct_generics),*>
            where
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                fn clone(&self) -> Self {
                    *self
                }
            }
        );

        let inherent_method = quote_spanned!(self.span =>
            pub fn #rust_field_name() -> impl #output_trait
            where
                #(#sig_where_clauses,)*
            {
                #field_struct

                #jvmop_impl

                #rust_field_type_name {
                    phantom: ::core::default::Default::default(),
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
                    let #input_name = duchess::prelude::AsJRef::as_jref(&#input_name)?;
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
    quote_spanned!(span => unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked(#byte_string) })
}
