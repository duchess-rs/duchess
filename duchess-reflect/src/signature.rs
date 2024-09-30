use crate::class_info::{
    ClassRef, Generic, Id, Method, NonRepeatingType, RefType, ScalarType, Type,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};

/// "Signature" processes Java argument/return types and
/// converts them into Rust types. This includes translating Java
/// generics into Rust generics.
pub struct Signature {
    /// Member being translated.
    item_name: Id,

    /// Span to use for error reporting.
    span: Span,

    /// Generic parameters from class, method.
    /// Used to check for validity and to avoid generating conflicting names.
    in_scope_generics: Vec<Id>,

    /// Generics to include on the generated Rust method.
    /// Includes the Java generics but also includes fresh generic
    /// parameters generated from Java wildcards.
    ///
    /// For example:
    ///
    /// ```java
    /// void foo(Class<?> c)
    /// ```
    ///
    /// would generate a Rust method like `fn foo<C>(c: Class<C>)`
    pub rust_generics: Vec<Ident>,

    /// Where clauses to include on the generated Rust method.
    pub where_clauses: Vec<TokenStream>,

    /// If true, permit `?` and translate to fresh generics.
    /// If false, report an error if `?` appears, because it is a context where
    /// we don't support capture.
    capture_generics: bool,
}

impl Signature {
    /// Creates a signature attached to an item (e.g., a method) named `method_name`,
    /// declared at `span`, which inherits `external_generics` from its class.
    ///
    /// You can then invoke helper methods to convert java types into Rust types.
    /// In some cases these conversions may create new entries in `fresh_generics`
    /// or new entries in `where_clauses`.
    ///
    /// The final Rust method needs to include all the parameters from `generics`
    /// along with the where-clauses from `where_clauses`.
    pub fn new(method_name: &Id, span: Span, external_generics: &[Generic]) -> Self {
        Signature {
            item_name: method_name.clone(),
            span,
            in_scope_generics: external_generics.iter().map(|g| g.id.clone()).collect(),
            rust_generics: vec![],
            where_clauses: vec![],
            capture_generics: true,
        }
    }

    /// Declares the generic parameters on the method/constructor being translated.
    /// "Internal" generics are distinct from "external" generics because they are
    /// added to `self.rust_generics` and `self.where_clauses`, so that in the end
    /// the caller has a unified list of all the generics that have to be declared on
    /// the Rust method.
    pub fn with_internal_generics(self, internal_generics: &[Generic]) -> syn::Result<Self> {
        let mut s = self;

        s.in_scope_generics
            .extend(internal_generics.iter().map(|g| g.id.clone()));

        // Forbid capture we don't have to worry about things like `X extends ArrayList<?>`.
        // Actually, we could probably support capture here, but I don't know want to right now.
        s.forbid_capture(|s| {
            for g in internal_generics {
                let ident = g.id.to_ident(s.span);
                s.rust_generics.push(ident.clone());
                s.where_clauses
                    .push(quote_spanned!(s.span => #ident : duchess::JavaObject));
                for e in &g.extends {
                    let ty = s.class_ref_ty_rs(e)?;
                    s.where_clauses
                        .push(quote_spanned!(s.span => #ident : duchess::AsJRef<#ty>));
                }
            }
            Ok::<(), syn::Error>(())
        })?;

        Ok(s)
    }

    /// Set the `capture_generics` field to false while `op` executes,
    /// then restore its value.
    pub fn forbid_capture<R>(&mut self, op: impl FnOnce(&mut Self) -> R) -> R {
        let v = std::mem::replace(&mut self.capture_generics, false);
        let r = op(self);
        self.capture_generics = v;
        r
    }

    /// Create and return a tuple with three fields:
    ///
    /// * the `input_ty_tts` token trees describing the input types to `method` (see [`Self::java_ty_tt`][])
    /// * the `input_ty_ops` token trees describing the JVM traits for each input type (see [`Self::jvm_op_trait`][])
    /// * the `input_names` identifiers naming each argument to `method`
    /// * the `output_ty_tt` token tree describing the return types of `method` (see [`Self::output_ty_tt`][])
    pub fn method_tts(
        &mut self,
        method: &Method,
        span: Span,
    ) -> syn::Result<(
        Vec<TokenStream>,
        Vec<TokenStream>,
        Vec<syn::Ident>,
        TokenStream,
    )> {
        let input_ty_tts = method
            .argument_tys
            .iter()
            .map(|ty| self.java_ty_tt(ty))
            .collect::<syn::Result<Vec<_>>>()?;

        let input_ty_ops = method
            .argument_tys
            .iter()
            .zip(&input_ty_tts)
            .map(|(ty, tt)| self.jvm_op_trait(ty, tt))
            .collect::<syn::Result<Vec<_>>>()?;

        let input_names: Vec<_> = (0..input_ty_tts.len())
            .map(|i| Ident::new(&format!("a{i}"), span))
            .collect();

        let output_ty_tt = self.output_ty_tt(&method.return_ty)?;

        Ok((input_ty_tts, input_ty_ops, input_names, output_ty_tt))
    }

    /// Generates a fresh generic type and adds it to `self.generics`.
    ///
    /// Used to manage Java wildcards. A type like `ArrayList<?>` gets
    /// translated to a Rust type like `ArrayList<Pi>` for some fresh `Pi`.
    ///
    /// See also `Self::push_where_bound`.
    fn fresh_generic(&mut self) -> syn::Result<Ident> {
        if !self.capture_generics {
            let msg = format!("unsupported wildcards in `{}`", self.item_name);
            Err(syn::Error::new(self.span, msg))
        } else {
            let mut i = self.rust_generics.len();
            loop {
                let ident = Ident::new(&format!("Capture{}", i), self.span);
                if !self.rust_generics.contains(&ident) {
                    self.rust_generics.push(ident.clone());
                    self.where_clauses
                        .push(quote_spanned!(self.span => #ident : duchess::JavaObject));
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

    /// Returns the name of the jni accessor method suitable for calling a function
    /// that returns a value of type `ty`.
    pub fn jni_call_fn(&mut self, ty: &Option<Type>) -> syn::Result<Ident> {
        let f = match ty {
            Some(Type::Ref(_)) => "CallObjectMethodA",
            Some(Type::Repeat(_)) => {
                return Err(syn::Error::new(
                    self.span,
                    format!(
                        "unsupported repeating return type in method `{}`",
                        self.item_name
                    ),
                ))
            }
            Some(Type::Scalar(scalar)) => match scalar {
                ScalarType::Int => "CallIntMethodA",
                ScalarType::Long => "CallLongMethodA",
                ScalarType::Short => "CallShortMethodA",
                ScalarType::Byte => "CallByteMethodA",
                ScalarType::F64 => "CallDoubleMethodA",
                ScalarType::F32 => "CallFloatMethodA",
                ScalarType::Boolean => "CallBooleanMethodA",
                ScalarType::Char => "CallCharMethodA",
            },
            None => "CallVoidMethodA",
        };
        Ok(Ident::new(f, self.span))
    }

    /// Returns the name of the jni accessor method suitable for calling a
    /// static function that returns a value of type `ty`.
    pub fn jni_static_call_fn(&mut self, ty: &Option<Type>) -> syn::Result<Ident> {
        let f = match ty {
            Some(Type::Ref(_)) => "CallStaticObjectMethodA",
            Some(Type::Repeat(_)) => {
                let msg = format!(
                    "unsupported repeating return type in static method `{}`",
                    self.item_name
                );
                return Err(syn::Error::new(self.span, msg));
            }
            Some(Type::Scalar(scalar)) => match scalar {
                ScalarType::Int => "CallStaticIntMethodA",
                ScalarType::Long => "CallStaticLongMethodA",
                ScalarType::Short => "CallStaticShortMethodA",
                ScalarType::Byte => "CallStaticByteMethodA",
                ScalarType::F64 => "CallStaticDoubleMethodA",
                ScalarType::F32 => "CallStaticFloatMethodA",
                ScalarType::Boolean => "CallStaticBooleanMethodA",
                ScalarType::Char => "CallStaticCharMethodA",
            },
            None => "CallStaticVoidMethodA",
        };
        Ok(Ident::new(f, self.span))
    }

    /// Returns the name of the jni accessor method suitable for getting
    /// the value of a static field of type `ty`.
    pub fn jni_static_field_get_fn(&mut self, ty: &Type) -> syn::Result<Ident> {
        let f = match ty {
            Type::Ref(_) => "GetStaticObjectField",
            Type::Repeat(_) => {
                let msg = format!(
                    "unsupported repeating type in getter of static field `{}`",
                    self.item_name
                );
                return Err(syn::Error::new(self.span, msg));
            }
            Type::Scalar(scalar) => match scalar {
                ScalarType::Int => "GetStaticIntField",
                ScalarType::Long => "GetStaticLongField",
                ScalarType::Short => "GetStaticShortField",
                ScalarType::Byte => "GetStaticByteField",
                ScalarType::F64 => "GetStaticDoubleField",
                ScalarType::F32 => "GetStaticFloatField",
                ScalarType::Boolean => "GetStaticBooleanField",
                ScalarType::Char => "GetStaticCharField",
            },
        };
        Ok(Ident::new(f, self.span))
    }

    /// Returns a path to a suitable JVM trait alias
    /// (`duchess::JvmRefOp` or `duchess::JavaScalarOp`)
    /// to the type. This really *ought* to be done in
    /// macro-rules code (e.g., `$I: duchess::plumbing::jvm_op!($_ty)`)
    /// but we can't easily do that because macro invocations
    /// aren't allowed in that position.
    pub fn jvm_op_trait(&mut self, ty: &Type, ty_tt: &TokenStream) -> syn::Result<TokenStream> {
        match &ty.to_non_repeating() {
            NonRepeatingType::Ref(_) => Ok(quote!(
                duchess::plumbing::JvmRefOp<duchess::plumbing::rust_ty!(#ty_tt)>
            )),
            NonRepeatingType::Scalar(_) => Ok(quote!(
                duchess::plumbing::JvmScalarOp<duchess::plumbing::rust_ty!(#ty_tt)>
            )),
        }
    }

    /// Return a token tree that can be passed to the macro-rules macros
    /// to represent the output type of a function; this can include `void`.
    pub fn output_ty_tt(&mut self, ty: &Option<Type>) -> syn::Result<TokenStream> {
        match ty {
            Some(ty) => self.java_ty_tt(ty),
            None => Ok(quote!(void)),
        }
    }

    /// Returns a token tree that can be passed to the various
    /// macro-rules (see `macro_rules/src/java_types.rs`
    /// for a description of the format) representing the type
    /// of an input.
    ///
    /// NB: This function may modify `self` to add fresh generics
    /// and bounds into the signature, allowing for the translation
    /// of Java wildcards in some cases. For example, a function that
    /// takes `List<?>` can be translated to a Rust function taking
    /// `List<T>` where `T: JavaObject`.
    pub fn java_ty_tt(&mut self, ty: &Type) -> syn::Result<TokenStream> {
        match &ty.to_non_repeating() {
            NonRepeatingType::Ref(ty) => self.java_ref_ty_tt(ty),
            NonRepeatingType::Scalar(ty) => Ok(self.java_scalar_ty_tt(ty)),
        }
    }

    /// Returns a Rust type that corresponds to the Java type `ty`.
    ///
    /// Use this when generating Rust code; to pass into the macro-rules macros,
    /// prefer [`Self::java_ty_tt`][].
    pub fn java_ty_rs(&mut self, ty: &Type) -> syn::Result<TokenStream> {
        let tt = self.java_ty_tt(ty)?;
        Ok(quote!(duchess::plumbing::rust_ty!(#tt)))
    }

    /// Return tokens to create the Rust type for a RefType.
    fn java_ref_ty_rs(&mut self, ty: &RefType) -> syn::Result<TokenStream> {
        let tt = self.java_ref_ty_tt(ty)?;
        Ok(quote!(duchess::plumbing::rust_ty!(#tt)))
    }

    /// Return the token-tree for a RefType.
    fn java_ref_ty_tt(&mut self, ty: &RefType) -> syn::Result<TokenStream> {
        match ty {
            RefType::Class(ty) => Ok(self.class_ref_ty_tt(ty)?),
            RefType::Array(e) => {
                let e = self.java_ty_tt(e)?;
                Ok(quote!((array #e)))
            }
            RefType::TypeParameter(t) => {
                if self.in_scope_generics.contains(t) {
                    let t = t.to_ident(self.span);
                    Ok(quote!((generic #t)))
                } else {
                    let msg = format!(
                        "generic type parameter `{:?}` not among in-scope parameters: {:?}",
                        t, self.in_scope_generics
                    );
                    Err(syn::Error::new(self.span, msg))
                }
            }
            RefType::Extends(ty) => {
                let g = self.fresh_generic()?;
                let e = self.java_ref_ty_rs(ty)?;
                self.push_where_bound(quote_spanned!(self.span => #g : duchess::AsJRef<#e>));
                Ok(quote!((generic #g)))
            }
            RefType::Super(_) => {
                let g = self.fresh_generic()?;
                // FIXME: missing where bound, really
                Ok(quote!((generic #g)))
            }
            RefType::Wildcard => {
                let g = self.fresh_generic()?;
                Ok(quote!((generic #g)))
            }
        }
    }

    pub fn class_ref_ty_rs(&mut self, ty: &ClassRef) -> syn::Result<TokenStream> {
        let tt = self.class_ref_ty_tt(ty)?;
        Ok(quote!(duchess::plumbing::rust_ty!(#tt)))
    }

    fn class_ref_ty_tt(&mut self, ty: &ClassRef) -> syn::Result<TokenStream> {
        let ClassRef { name, generics } = ty;
        let rust_name = name.to_module_name(self.span);
        let rust_ty_tts: Vec<_> = generics
            .iter()
            .map(|t| self.java_ref_ty_tt(t))
            .collect::<Result<_, _>>()?;
        Ok(quote!((class[#rust_name] #(#rust_ty_tts)*)))
    }

    fn java_scalar_ty_tt(&self, ty: &ScalarType) -> TokenStream {
        match ty {
            ScalarType::Int => quote!(int),
            ScalarType::Long => quote!(long),
            ScalarType::Short => quote!(short),
            ScalarType::Byte => quote!(byte),
            ScalarType::F64 => quote!(double),
            ScalarType::F32 => quote!(float),
            ScalarType::Boolean => quote!(boolean),
            ScalarType::Char => quote!(char),
        }
    }
}
