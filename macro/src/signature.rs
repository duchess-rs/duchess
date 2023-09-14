use crate::class_info::{ClassRef, Generic, Id, NonRepeatingType, RefType, ScalarType, Type};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote_spanned;

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
                    let ty = s.class_ref_ty(e)?;
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

    /// Returns an appropriate `impl type` for a funtion that
    /// takes `ty` as input. Assumes objects are nullable.
    pub fn input_trait(&mut self, ty: &Type) -> syn::Result<TokenStream> {
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

    /// Returns an appropriate `impl type` for a function that
    /// returns a `ty` or void. Assumes objects are nullable.
    pub fn output_type(&mut self, ty: &Option<Type>) -> syn::Result<TokenStream> {
        match ty.as_ref() {
            Some(ty) => self.non_void_output_type(ty),
            None => Ok(quote_spanned!(self.span => ())),
        }
    }

    /// Returns an appropriate `impl type` for a function that
    /// returns `ty`. Assumes objects are nullable.
    pub fn non_void_output_type(&mut self, ty: &Type) -> syn::Result<TokenStream> {
        // XX: do we need the non_repeating transform here? Shouldn't be allowed in return position
        self.forbid_capture(|this| match ty.to_non_repeating() {
            NonRepeatingType::Ref(ty) => {
                let t = this.java_ref_ty(&ty)?;
                Ok(quote_spanned!(this.span => ::core::option::Option<duchess::Local<'jvm, #t>>))
            }
            NonRepeatingType::Scalar(ty) => {
                let t = this.java_scalar_ty(&ty);
                Ok(quote_spanned!(this.span => #t))
            }
        })
    }

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

    /// Returns an appropriate trait for a method that
    /// returns `ty`. Assumes objects are nullable.
    pub fn method_trait(&mut self, ty: &Option<Type>) -> syn::Result<TokenStream> {
        self.forbid_capture(|this| match ty.as_ref().map(|ty| ty.to_non_repeating()) {
            Some(NonRepeatingType::Ref(ty)) => {
                let t = this.java_ref_ty(&ty)?;
                Ok(quote_spanned!(this.span => duchess::JavaMethod<#t>))
            }
            Some(NonRepeatingType::Scalar(ty)) => {
                let t = this.java_scalar_ty(&ty);
                Ok(quote_spanned!(this.span => duchess::ScalarMethod<#t>))
            }
            None => Ok(quote_spanned!(this.span => duchess::VoidMethod)),
        })
    }

    /// Returns an appropriate trait for a field that
    /// returns `ty`. Assumes objects are nullable.
    pub fn field_trait(&mut self, ty: &Type) -> syn::Result<TokenStream> {
        self.forbid_capture(|this| match ty.to_non_repeating() {
            NonRepeatingType::Ref(ty) => {
                let t = this.java_ref_ty(&ty)?;
                Ok(quote_spanned!(this.span => duchess::JavaField<#t>))
            }
            NonRepeatingType::Scalar(ty) => {
                let t = this.java_scalar_ty(&ty);
                Ok(quote_spanned!(this.span => duchess::ScalarField<#t>))
            }
        })
    }

    /// Returns the Rust type that represents this Java type -- e.g., for `Object`,
    /// returns `java::lang::Object`. Note that this is not the type of a *reference* to this
    /// java type (which would be e.g. `Global<java::lang::Object>`).
    pub fn java_ty(&mut self, ty: &Type) -> syn::Result<TokenStream> {
        match &ty.to_non_repeating() {
            NonRepeatingType::Ref(ty) => self.java_ref_ty(ty),
            NonRepeatingType::Scalar(ty) => Ok(self.java_scalar_ty(ty)),
        }
    }

    /// Like `java_ty`, but only for reference types (returns `None` for scalars).
    pub fn java_ty_if_ref(&mut self, ty: &Type) -> syn::Result<Option<TokenStream>> {
        match &ty.to_non_repeating() {
            NonRepeatingType::Ref(ty) => Ok(Some(self.java_ref_ty(ty)?)),
            NonRepeatingType::Scalar(_) => Ok(None),
        }
    }

    fn java_ref_ty(&mut self, ty: &RefType) -> syn::Result<TokenStream> {
        match ty {
            RefType::Class(ty) => Ok(self.class_ref_ty(ty)?),
            RefType::Array(e) => {
                let e = self.java_ty(e)?;
                Ok(quote_spanned!(self.span => java::Array<#e>))
            }
            RefType::TypeParameter(t) => {
                if self.in_scope_generics.contains(t) {
                    let t = t.to_ident(self.span);
                    Ok(quote_spanned!(self.span => #t))
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
                let e = self.java_ref_ty(ty)?;
                self.push_where_bound(quote_spanned!(self.span => #g : duchess::AsJRef<#e>));
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

    pub fn class_ref_ty(&mut self, ty: &ClassRef) -> syn::Result<TokenStream> {
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
        ty.to_tokens(self.span)
    }
}
