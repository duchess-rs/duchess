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
use quote::{quote, quote_spanned};

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
        let java_class_generics = self.class_generic_names();
        let jni_class_name = self.jni_class_name();

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

        let op_name = Id::from(format!("ViewAs{}Op", self.name.class_name())).to_ident(self.span);
        let obj_name = Id::from(format!("ViewAs{}Obj", self.name.class_name())).to_ident(self.span);

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

        let mro_tys = self.mro(upcasts)?;

        let output = quote! {
            duchess::plumbing::setup_class! {
                struct_name: [#struct_name],
                java_class_generics: [#(#java_class_generics,)*],
                jni_class_name: [#jni_class_name],
                mro_tys: [#(#mro_tys,)*],
                constructors: [#(#constructors)*],
                static_methods: [#(#static_methods)*],
                static_field_getters: [#(#static_field_getters)*],
                inherent_object_methods: [#(#inherent_object_methods)*],
                op_struct_methods: [#(#op_methods)*],
                obj_struct_methods: [#(#obj_methods)*],
                op_name: [#op_name],
                obj_name: [#obj_name],
            }
        };

        crate::debug_tokens(&self.name, &output);

        Ok(output)
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

    fn constructor(&self, constructor: &Constructor) -> syn::Result<TokenStream> {
        let mut sig = Signature::new(self.name.class_name(), self.span, &self.generics);

        let (input_traits, jvm_op_traits): (Vec<_>, Vec<_>) = constructor
            .argument_tys
            .iter()
            .map(|ty| sig.input_and_jvm_op_traits(ty))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

        let input_names: Vec<_> = (0..input_traits.len())
            .map(|i| Ident::new(&format!("a{i}"), self.span))
            .collect();

        let struct_name = self.struct_name();

        let java_class_generics = self.class_generic_names();

        let jni_descriptor = jni_c_str(constructor.descriptor(&self.generics_scope()), self.span);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &constructor.argument_tys);

        // for debugging JVM invocation failures
        let descriptor = Literal::string(&constructor.descriptor(&self.generics_scope()));

        Ok(quote! {
            duchess::plumbing::setup_constructor! {
                struct_name: [#struct_name],
                java_class_generics: [#(#java_class_generics,)*],
                input_names: [#(#input_names,)*],
                input_traits: [#(#input_traits,)*],
                jvm_op_traits: [#(#jvm_op_traits,)*],
                prepare_inputs: [#(#prepare_inputs)*],
                descriptor: [#descriptor],
                jni_descriptor: [#jni_descriptor],
                idents: [self, jvm],
            }
        })
    }

    /// Generates code for the methods that goes on the `ops` object.
    ///
    ///
    /// NB. This function (particularly the JvmOp impl) has significant overlap with `static_method`
    /// and `static_field_getter`, so if you make changes here, you may well need changes there.
    fn op_struct_method(&self, method: &Method) -> syn::Result<TokenStream> {
        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();

        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let (input_traits, _jvm_op_traits): (Vec<_>, Vec<_>) = method
            .argument_tys
            .iter()
            .map(|ty| sig.input_and_jvm_op_traits(ty))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

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

        Ok(quote!(duchess::plumbing::setup_op_method! {
            struct_name: [#struct_name],
            java_class_generics: [#(#java_class_generics,)*],
            rust_method_name: [#rust_method_name],
            rust_method_generics: [#(#rust_method_generics,)*],
            input_names: [#(#input_names,)*],
            input_traits: [#(#input_traits,)*],
            output_trait: [#output_trait],
            sig_where_clauses: [#(#sig_where_clauses,)*],
        }))
    }

    fn obj_struct_method(&self, method: &Method) -> syn::Result<TokenStream> {
        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();

        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let (input_traits, _jvm_op_traits): (Vec<_>, Vec<_>) = method
            .argument_tys
            .iter()
            .map(|ty| sig.input_and_jvm_op_traits(ty))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

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

        Ok(quote!(duchess::plumbing::setup_obj_method! {
            struct_name: [#struct_name],
            java_class_generics: [#(#java_class_generics,)*],
            rust_method_name: [#rust_method_name],
            rust_method_generics: [#(#rust_method_generics,)*],
            input_names: [#(#input_names,)*],
            input_traits: [#(#input_traits,)*],
            output_trait: [#output_trait],
            sig_where_clauses: [#(#sig_where_clauses,)*],
        }))
    }

    fn inherent_object_method(&self, method: &Method) -> syn::Result<TokenStream> {
        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let (input_traits, jvm_op_traits): (Vec<_>, Vec<_>) = method
            .argument_tys
            .iter()
            .map(|ty| sig.input_and_jvm_op_traits(ty))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

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

        let jni_descriptor = jni_c_str(&method.descriptor(&self.generics_scope()), self.span);

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
            impl<#(#method_struct_generics),*> ::core::clone::Clone
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #this: duchess::plumbing::JvmRefOp<#this_ty>,
                #(#input_names: #jvm_op_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                fn clone(&self) -> Self {
                    #rust_method_type_name {
                        #this: Clone::clone(&self.#this),
                        #(#input_names: Clone::clone(&self.#input_names),)*
                        phantom: self.phantom,
                    }
                }
            }

            impl<#(#method_struct_generics),*> duchess::prelude::JvmOp
            for #rust_method_type_name<#(#method_struct_generics),*>
            where
                #this: duchess::plumbing::JvmRefOp<#this_ty>,
                #(#input_names: #jvm_op_traits,)*
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                type Output<'jvm> = #output_ty;

                fn do_jni<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::LocalResult<'jvm, Self::Output<'jvm>> {
                    let this = self.#this.into_as_jref(jvm)?;
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
                        jvm.env().invoke(|env| env.#jni_call_fn, |env, f| f(
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
                    #this: #this.into_op(),
                    #(#input_names: #input_names.into_op(),)*
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

        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();

        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let (input_traits, jvm_op_traits): (Vec<_>, Vec<_>) = method
            .argument_tys
            .iter()
            .map(|ty| sig.input_and_jvm_op_traits(ty))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

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

        let jni_descriptor = jni_c_str(&method.descriptor(&self.generics_scope()), self.span);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &method.argument_tys);

        let jni_method = jni_c_str(&*method.name, self.span);

        let rust_method_name = Id::from(method.name.to_snake_case()).to_ident(self.span);
        let rust_method_struct_name = Id::from(method.name.to_camel_case()).to_ident(self.span);

        // The generic parameters we need on the Rust method, these include:
        //
        // * a type parameter for each java generic
        // * any fresh generics we created to capture wildcards
        let rust_method_generics = &sig.rust_generics;

        let sig_where_clauses = &sig.where_clauses;

        Ok(quote!(duchess::plumbing::setup_static_method! {
            struct_name: [#struct_name],
            java_class_generics: [#(#java_class_generics,)*],
            rust_method_name: [#rust_method_name],
            rust_method_struct_name: [#rust_method_struct_name],
            rust_method_generics: [#(#rust_method_generics,)*],
            input_names: [#(#input_names,)*],
            input_traits: [#(#input_traits,)*],
            jvm_op_traits: [#(#jvm_op_traits,)*],
            output_ty: [#output_ty],
            output_trait: [#output_trait],
            java_ref_output_ty: [#java_ref_output_ty],
            sig_where_clauses: [#(#sig_where_clauses,)*],
            prepare_inputs: [#(#prepare_inputs)*],
            jni_call_fn: [#jni_call_fn],
            jni_method: [#jni_method],
            jni_descriptor: [#jni_descriptor],
            idents: [self, jvm],
        }))
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
        let jni_descriptor = jni_c_str(&field.ty.descriptor(&self.generics_scope()), self.span);

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

                fn do_jni<'jvm>(
                    self,
                    jvm: &mut duchess::Jvm<'jvm>,
                ) -> duchess::LocalResult<'jvm, Self::Output<'jvm>> {

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
                        jvm.env().invoke(|env| env.#jni_field_fn, |env, f| f(
                            env,
                            duchess::plumbing::JavaObjectExt::as_raw(&*class).as_ptr(),
                            field.as_ptr(),
                        ))
                    }
                }
            }

            impl<#(#field_struct_generics),*> ::core::clone::Clone for #rust_field_type_name<#(#field_struct_generics),*>
            where
                #(#java_class_generics: duchess::JavaObject,)*
                #(#sig_where_clauses,)*
            {
                fn clone(&self) -> Self {
                    #rust_field_type_name {
                        phantom: self.phantom,
                    }
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
                    let #input_name = self.#input_name.do_jni(jvm)?;
                ),
                NonRepeatingType::Ref(_) => {
                    quote_spanned!(self.span =>
                        let #input_name = self.#input_name.into_as_jref(jvm)?;
                        let #input_name = match duchess::prelude::AsJRef::as_jref(&#input_name) {
                            Ok(v) => Some(v),
                            Err(duchess::NullJRef) => None,
                        };
                    )
                }
            })
            .collect()
    }
}

fn jni_c_str(contents: impl Into<String>, span: Span) -> TokenStream {
    let mut contents = contents.into().into_bytes();
    // \0 isn't valid UTF-8, so don't need to check that contents doesn't contain interior nul bytes.
    contents.push(0);

    let byte_string = Literal::byte_string(&contents);
    quote_spanned!(span => unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked(#byte_string) })
}
