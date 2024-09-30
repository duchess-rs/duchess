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
                Ok(sig.forbid_capture(|sig| sig.class_ref_ty_rs(r)).unwrap())
            })
            .collect()
    }

    fn constructor(&self, constructor: &Constructor) -> syn::Result<TokenStream> {
        let mut sig = Signature::new(self.name.class_name(), self.span, &self.generics);

        let input_ty_tts = constructor
            .argument_tys
            .iter()
            .map(|ty| sig.java_ty_tt(ty))
            .collect::<syn::Result<Vec<_>>>()?;

        let input_ty_ops = constructor
            .argument_tys
            .iter()
            .zip(&input_ty_tts)
            .map(|(ty, tt)| sig.jvm_op_trait(ty, tt))
            .collect::<syn::Result<Vec<_>>>()?;

        let input_names: Vec<Ident> = (0..input_ty_tts.len())
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
                input_ty_tts: [#(#input_ty_tts,)*],
                input_ty_ops: [#(#input_ty_ops,)*],
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

        let (input_ty_tts, _input_ty_ops, input_names, output_ty_tt) =
            sig.method_tts(method, self.span)?;

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
            input_ty_tts: [#(#input_ty_tts,)*],
            output_ty_tt: [#output_ty_tt],
            sig_where_clauses: [#(#sig_where_clauses,)*],
        }))
    }

    fn obj_struct_method(&self, method: &Method) -> syn::Result<TokenStream> {
        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();

        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let (input_ty_tts, _input_ty_ops, input_names, output_ty_tt) =
            sig.method_tts(method, self.span)?;

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
            input_ty_tts: [#(#input_ty_tts,)*],
            output_ty_tt: [#output_ty_tt],
            sig_where_clauses: [#(#sig_where_clauses,)*],
        }))
    }

    fn inherent_object_method(&self, method: &Method) -> syn::Result<TokenStream> {
        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();
        let mut sig = Signature::new(&method.name, self.span, &self.generics)
            .with_internal_generics(&method.generics)?;

        let (input_ty_tts, input_ty_ops, input_names, output_ty_tt) =
            sig.method_tts(method, self.span)?;

        // The appropriate JNI function to call this method.
        let jni_call_fn = sig.jni_call_fn(&method.return_ty)?;

        let jni_descriptor = jni_c_str(&method.descriptor(&self.generics_scope()), self.span);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &method.argument_tys);

        let jni_method = jni_c_str(&*method.name, self.span);

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

        Ok(quote!(duchess::plumbing::setup_inherent_object_method! {
            struct_name: [#struct_name],
            java_class_generics: [#(#java_class_generics,)*],
            rust_method_name: [#rust_method_name],
            rust_method_generics: [#(#rust_method_generics,)*],
            input_names: [#(#input_names,)*],
            input_ty_tts: [#(#input_ty_tts,)*],
            input_ty_ops: [#(#input_ty_ops,)*],
            output_ty_tt: [#output_ty_tt],
            sig_where_clauses: [#(#sig_where_clauses,)*],
            prepare_inputs: [#(#prepare_inputs)*],
            jni_call_fn: [#jni_call_fn],
            jni_method: [#jni_method],
            jni_descriptor: [#jni_descriptor],
            idents: [self, jvm],
        }))
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

        let (input_ty_tts, input_ty_ops, input_names, output_ty_tt) =
            sig.method_tts(method, self.span)?;

        let jni_call_fn = sig.jni_static_call_fn(&method.return_ty)?;

        let jni_descriptor = jni_c_str(&method.descriptor(&self.generics_scope()), self.span);

        // Code to convert each input appropriately
        let prepare_inputs = self.prepare_inputs(&input_names, &method.argument_tys);

        let jni_method = jni_c_str(&*method.name, self.span);

        let rust_method_name = Id::from(method.name.to_snake_case()).to_ident(self.span);

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
            rust_method_generics: [#(#rust_method_generics,)*],
            input_names: [#(#input_names,)*],
            input_ty_tts: [#(#input_ty_tts,)*],
            input_ty_ops: [#(#input_ty_ops,)*],
            output_ty_tt: [#output_ty_tt],
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

        let struct_name = self.struct_name();
        let java_class_generics = self.class_generic_names();

        let mut sig = Signature::new(&field.name, self.span, &self.generics);

        let field_ty = sig.java_ty_tt(&field.ty)?;

        let jni_field = jni_c_str(&*field.name, self.span);
        let jni_descriptor = jni_c_str(&field.ty.descriptor(&self.generics_scope()), self.span);

        let rust_field_name =
            Id::from(format!("get_{}", field.name.to_snake_case())).to_ident(self.span);

        let sig_where_clauses = &sig.where_clauses;

        Ok(quote!(duchess::plumbing::setup_static_field_getter! {
            struct_name: [#struct_name],
            java_class_generics: [#(#java_class_generics,)*],
            rust_field_name: [#rust_field_name],
            field_ty: [#field_ty],
            sig_where_clauses: [#(#sig_where_clauses,)*],
            jni_field: [#jni_field],
            jni_descriptor: [#jni_descriptor],
        }))
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
