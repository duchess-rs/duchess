use std::collections::VecDeque;

use proc_macro2::Span;
use quote::quote_spanned;
use rust_format::Formatter;
use syn::{spanned::Spanned, Attribute};
use synstructure::VariantInfo;

use crate::{argument::MethodSelector, parse::Parser, reflect::Reflector, signature::Signature};

pub fn derive_to_rust(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let mut driver = Driver {
        input: &s,
        reflector: &mut Reflector::default(),
    };
    match driver.try_derive_to_rust() {
        Ok(t) => {
            if let Ok(debug_env) = std::env::var("DUCHESS_DEBUG") {
                if debug_env == "*" || debug_env == "1" || s.ast().ident.to_string() == debug_env {
                    match rust_format::RustFmt::default().format_tokens(t.clone()) {
                        Ok(v) => {
                            eprintln!("{v}");
                        }
                        Err(_) => {
                            eprintln!("{t}");
                        }
                    }
                }
            }

            t
        }
        Err(e) => e.into_compile_error(),
    }
}

pub fn derive_to_java(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let mut driver = Driver {
        input: &s,
        reflector: &mut Reflector::default(),
    };
    match driver.try_derive_to_java() {
        Ok(t) => {
            if let Ok(debug_env) = std::env::var("DUCHESS_DEBUG") {
                if debug_env == "*" || debug_env == "1" || s.ast().ident.to_string() == debug_env {
                    match rust_format::RustFmt::default().format_tokens(t.clone()) {
                        Ok(v) => {
                            eprintln!("{v}");
                        }
                        Err(_) => {
                            eprintln!("{t}");
                        }
                    }
                }
            }

            t
        }
        Err(e) => e.into_compile_error(),
    }
}

struct Driver<'a> {
    input: &'a synstructure::Structure<'a>,
    reflector: &'a mut Reflector,
}

impl Driver<'_> {
    fn try_derive_to_rust(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        match self.input.ast().data {
            syn::Data::Struct(_) => self.try_derive_to_rust_struct(),
            syn::Data::Enum(_) => {
                return Err(syn::Error::new(self.span(), "enums not (yet) supported"));
            }
            syn::Data::Union(_) => {
                return Err(syn::Error::new(self.span(), "unions not supported"));
            }
        }
    }

    fn try_derive_to_java(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        match self.input.ast().data {
            syn::Data::Struct(_) => self.try_derive_to_java_struct(),
            syn::Data::Enum(_) => {
                return Err(syn::Error::new(self.span(), "enums not (yet) supported"));
            }
            syn::Data::Union(_) => {
                return Err(syn::Error::new(self.span(), "unions not supported"));
            }
        }
    }

    fn span(&self) -> Span {
        self.input.ast().ident.span()
    }

    fn try_derive_to_rust_struct(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let variants = self.input.variants();
        let variant = &variants[0];
        let variant_span = variant.ast().ident.span();

        if !self.input.ast().generics.params.is_empty() {
            // FIXME
            return Err(syn::Error::new(
                self.span(),
                "generic structs not yet supported",
            ));
        }

        let to_rust_body = self.variant_to_rust(variant)?;

        let method_selector = self.find_method_selector(variant_span, variant.ast().attrs)?;
        let reflected_method = self.reflector.reflect_method(&method_selector)?;
        let class_name = reflected_method
            .class()
            .name
            .to_module_name(method_selector.class_span());

        let ext_trait_name = reflected_method
            .class()
            .name
            .to_ext_trait_name(method_selector.class_span());

        let self_ty = &self.input.ast().ident;
        Ok(quote_spanned!(self.span() =>
        impl duchess::ToRust<#self_ty> for #class_name {
            fn to_rust<'jvm>(&self, jvm: &mut duchess::Jvm<'jvm>) -> duchess::Result<'jvm, #self_ty> {
                use #ext_trait_name;
                Ok(#to_rust_body)
            }
        }
        ))
    }

    fn try_derive_to_java_struct(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let variants = self.input.variants();
        let variant = &variants[0];
        let variant_span = variant.ast().ident.span();

        if !self.input.ast().generics.params.is_empty() {
            // FIXME
            return Err(syn::Error::new(
                self.span(),
                "generic structs not yet supported",
            ));
        }

        let to_java_body = self.variant_to_java(variant)?;

        let method_selector = self.find_method_selector(variant_span, variant.ast().attrs)?;
        let reflected_method = self.reflector.reflect_method(&method_selector)?;
        let class_name = reflected_method
            .class()
            .name
            .to_module_name(method_selector.class_span());

        let self_ty = &self.input.ast().ident;
        Ok(quote_spanned!(self.span() =>
            impl duchess::JvmOp for & #self_ty {
                type Output<'jvm> = Local<'jvm, #class_name>;

                fn execute_with<'jvm>(self, jvm: &mut duchess::Jvm<'jvm>) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                    match self {
                        #to_java_body
                    }
                }
            }

            impl duchess::plumbing::ToJavaImpl<#class_name> for #self_ty {
                fn to_java_impl<'jvm>(rust: &Self, jvm: &mut duchess::Jvm<'jvm>) -> duchess::Result<'jvm, Option<Local<'jvm, #class_name>>> {
                    Ok(Some(duchess::JvmOp::execute_with(rust, jvm)?))
                }
            }
        ))
    }

    /// Generates the code to create this variant as part of a `ToRust` impl.
    /// Assumes `self` is the java type and `jvm` is in scope.
    fn variant_to_rust(
        &mut self,
        variant: &VariantInfo,
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        // For each field, construct an expression we will use to initialize its value.
        let mut initializers = VecDeque::new();
        for field in variant.ast().fields {
            if let Some(name) = &field.ident {
                if name == "this" {
                    // Special case for fields named this
                    initializers
                        .push_back(quote_spanned!(name.span() => self.global().execute_with(jvm)?));
                } else if self.is_option(&field.ty) {
                    initializers.push_back(quote_spanned!(name.span() =>
                    self
                        .#name()
                        .to_rust()
                        .execute_with(jvm)?
                    ));
                } else {
                    initializers.push_back(quote_spanned!(name.span() =>
                    self
                        .#name()
                        .assert_not_null()
                        .to_rust()
                        .execute_with(jvm)?
                    ));
                }
            } else {
                // FIXME: We should probably support something like
                // `#[duchess::args(foo, bar, bar)]` ?
                return Err(syn::Error::new(
                    field.span(),
                    "tuple structs not yet supported",
                ));
            }
        }

        let mut counter = 0;
        let construct = variant.construct(|_field, index| {
            assert!(counter == index);
            counter += 1;
            initializers.pop_front().unwrap()
        });

        Ok(construct)
    }

    /// Generates the code to create this variant as part of a `ToJava` impl.
    /// Assumes `self` is the java type and `jvm` is in scope.
    fn variant_to_java(
        &mut self,
        variant: &VariantInfo,
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        let variant_span = variant.ast().ident.span();

        // If there is a field named `this`, just return that.
        if let Some(binding) = variant
            .bindings()
            .iter()
            .find(|b| b.ast().ident.as_ref().map(|i| i == "this").unwrap_or(false))
        {
            let pattern = variant.pat();
            return Ok(quote_spanned!(self.span() =>
                #pattern => {
                    Ok(jvm.local(#binding))
                }
            ));
        }

        // Otherwise, we will construct a call to `java::package::Class::new` where
        // the arguments are taken from each field. One challenge is that we have to
        // know the constructor so we can find the expected types, since we need
        // to provide those when we call `.to_java::<J>()`.

        let method_selector = self.find_method_selector(variant_span, variant.ast().attrs)?;

        let reflected_method = self.reflector.reflect_method(&method_selector)?;

        if !reflected_method.is_static() {
            return Err(syn::Error::new(
                method_selector.span(),
                "selected method is not a constructor or a static method",
            ));
        }

        // We are going to pass each field as an argument to the method,
        // so there have to be the same number.
        //
        // FIXME: Variadic methods in Java?
        let method_arguments = reflected_method.argument_tys();
        if method_arguments.len() != variant.ast().fields.len() {
            return Err(syn::Error::new(
                method_selector.span(),
                format!(
                    "selected method or constructor has {} arguments, but there are {} fields",
                    method_arguments.len(),
                    variant.ast().fields.len()
                ),
            ));
        }

        // We don't (yet?) support methods with generic arguments, because we'd have to figure
        // out what their value should be so that we can specify them as part of the `.to_java::<J>()`
        // calls.
        //
        // FIXME: We could allow users to tell us, I guess.
        if reflected_method.generics().len() != 0 {
            return Err(syn::Error::new(
                method_selector.span(),
                format!("selected method or constructor has generic parameters, not supported",),
            ));
        }

        let mut signature = Signature::new(
            &reflected_method.name(),
            method_selector.span(),
            &reflected_method.class().generics,
        );

        let java_types: Vec<_> = signature.forbid_capture(|signature| {
            method_arguments
                .iter()
                .map(|t| signature.java_ty(t))
                .collect::<Result<_, _>>()
        })?;

        let args: Vec<_> = variant
            .bindings()
            .iter()
            .zip(&java_types)
            .map(|(binding, java_type)| {
                quote_spanned!(binding.span() =>
                    duchess::ToJava::to_java::<#java_type>(#binding)
                )
            })
            .collect();

        let class_name = reflected_method
            .class()
            .name
            .to_module_name(method_selector.class_span());
        let method_name = reflected_method.name().to_ident(method_selector.span());

        let pattern = variant.pat();
        Ok(quote_spanned!(self.span() =>
            #pattern => {
                #class_name :: #method_name ( #(#args),* ) . execute_with(jvm)
            }
        ))
    }

    fn find_method_selector(
        &mut self,
        span: Span,
        attrs: &[Attribute],
    ) -> Result<MethodSelector, syn::Error> {
        for attr in attrs {
            let path = attr.meta.path();
            if path.is_ident("java") {
                let list = attr.meta.require_list()?;
                if let syn::MacroDelimiter::Paren(_) = list.delimiter {
                    return Ok(Parser::from(list.tokens.clone()).parse()?);
                };
                return Err(syn::Error::new(
                    attr.span(),
                    r#"expected `#[java(class.name)`]"#,
                ));
            }
        }
        return Err(syn::Error::new(
            span,
            r#"supply a `#[java(class.name)` to indicate the java class"#,
        ));
    }

    fn is_option(&self, ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Path(p) => p.path.is_ident("Option"),
            _ => false,
        }
    }
}
