use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{spanned::Spanned, Attribute};
use synstructure::VariantInfo;

use crate::{
    argument::{JavaPath, MethodSelector},
    class_info::{ClassInfo, ClassRef, Type},
    parse::{Parse, Parser},
    reflect::Reflector,
    signature::Signature,
    upcasts::Upcasts,
};

pub fn derive_to_rust(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let mut driver = Driver {
        input: &s,
        reflector: &mut Reflector::default(),
    };
    match driver.try_derive_to_rust() {
        Ok(t) => {
            crate::debug_tokens(&s.ast().ident, &t);
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
            crate::debug_tokens(&s.ast().ident, &t);
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
        self.check_generics()?;

        match self.input.ast().data {
            syn::Data::Struct(_) => self.try_derive_to_rust_struct(),
            syn::Data::Enum(_) => self.try_derive_to_rust_enum(),
            syn::Data::Union(_) => {
                return Err(syn::Error::new(self.span(), "unions not supported"));
            }
        }
    }

    fn try_derive_to_java(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        self.check_generics()?;

        match self.input.ast().data {
            syn::Data::Struct(_) => self.try_derive_to_java_struct(),
            syn::Data::Enum(_) => self.try_derive_to_java_enum(),
            syn::Data::Union(_) => {
                return Err(syn::Error::new(self.span(), "unions not supported"));
            }
        }
    }

    fn span(&self) -> Span {
        self.input.ast().ident.span()
    }

    fn try_derive_to_rust_struct(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let variants = self.to_rust_variants()?;
        assert_eq!(variants.len(), 1);
        self.try_derive_to_rust_variants(&variants[0], &[/* children */])
    }

    fn try_derive_to_rust_enum(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let root_path: JavaPath = self.find_java_attr(self.span(), &self.input.ast().attrs)?;
        let variants = self.to_rust_variants()?;
        let upcasts: Upcasts = variants.iter().map(|v| &*v.class).collect();

        let variant_classes = unique_variant_classes(&variants)?;
        let variants = order_by_specificity(&variant_classes, &upcasts);
        // Root must be upcast of all children, so must come last when ordered by specificity!
        let Some((&root, children)) = variants.split_last() else {
            return Err(syn::Error::new(self.span(), "enum must have at least one variant"));
        };
        if root.class.name != root_path.to_dot_id() {
            return Err(syn::Error::new(
                root_path.span,
                format!("must have one enum variant for root Java class `{root_path}`"),
            ));
        }
        // XX: If one of the classes in a variant's real upcast chain isn't included in the enum
        // then the upcast chain will be missing chunks from our perspective!
        check_all_extend_root(&root.class, children.iter().map(|c| &c.selector), &upcasts)?;

        self.try_derive_to_rust_variants(root, children)
    }

    // Emits an `impl ToRust` as a chain of `if try_downcast()` for child variants (if any) followed by an `else` for
    // the root variant.
    fn try_derive_to_rust_variants(
        &self,
        root: &ToRustVariant<'_>,
        children: &[&ToRustVariant<'_>],
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        let root_class_name = root.class.name.to_module_name(root.selector.span());
        let root_to_rust = self.variant_to_rust(
            quote_spanned!(root.variant.ast().ident.span() => self),
            root.variant,
        )?;

        let child_class_names = children
            .iter()
            .map(|c| c.class.name.to_module_name(c.selector.span()))
            .collect::<Vec<_>>();
        let child_to_rust = children
            .iter()
            .map(|c| {
                self.variant_to_rust(
                    quote_spanned!(c.variant.ast().ident.span() => variant),
                    c.variant,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let self_ty = &self.input.ast().ident;

        Ok(quote_spanned!(self.span() =>
        #[allow(unused_imports, unused_variables)]
        impl duchess::IntoRust<#self_ty> for &#root_class_name {
            fn into_rust<'jvm>(self, jvm: &mut duchess::Jvm<'jvm>) -> duchess::Result<'jvm, #self_ty> {
                use duchess::prelude::*;
                #(
                    if let Ok(variant) = self.try_downcast::<#child_class_names>().execute_with(jvm)? {
                        Ok(#child_to_rust)
                    } else
                )*
                {
                    Ok(#root_to_rust)
                }
            }
        }
        ))
    }

    fn try_derive_to_java_struct(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let variant = &self.input.variants()[0];
        let method = self.find_method_selector(variant.ast().ident.span(), variant.ast().attrs)?;
        let class = self
            .reflector
            .reflect(&method.class_name(), method.class_span())?;
        self.try_derive_to_java_variants(&class, [variant])
    }

    fn try_derive_to_java_enum(&mut self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let root_path: JavaPath = self.find_java_attr(self.span(), &self.input.ast().attrs)?;
        let root_class = self
            .reflector
            .reflect(&root_path.to_dot_id(), root_path.span)?;

        let selectors = self
            .input
            .variants()
            .iter()
            .map(|v| self.find_method_selector(v.ast().ident.span(), v.ast().attrs))
            .collect::<Result<Vec<_>, _>>()?;
        let classes = selectors
            .iter()
            .map(|s| self.reflector.reflect(&s.class_name(), s.class_span()))
            .collect::<Result<Vec<_>, _>>()?;
        let upcasts = classes.iter().map(|c| &**c).collect::<Upcasts>();
        check_all_extend_root(&root_class, selectors.iter(), &upcasts)?;

        self.try_derive_to_java_variants(&root_class, self.input.variants())
    }

    fn try_derive_to_java_variants<'a>(
        &self,
        root_class: &ClassInfo,
        variants: impl IntoIterator<Item = &'a VariantInfo<'a>>,
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        let root_class_name = root_class.name.to_module_name(root_class.span);

        let to_java_bodies = variants
            .into_iter()
            .map(|v| self.variant_to_java(v))
            .collect::<Result<Vec<_>, _>>()?;

        let self_ty = &self.input.ast().ident;
        Ok(quote_spanned!(self.span() =>
            #[allow(unused_imports, unused_variables)]
            impl duchess::JvmOp for & #self_ty {
                type Output<'jvm> = duchess::Local<'jvm, #root_class_name>;

                fn execute_with<'jvm>(self, jvm: &mut duchess::Jvm<'jvm>) -> duchess::Result<'jvm, Self::Output<'jvm>> {
                    use duchess::prelude::*;
                    match self {
                        #(#to_java_bodies),*
                    }
                }
            }

            impl duchess::plumbing::ToJavaImpl<#root_class_name> for #self_ty {
                fn to_java_impl<'jvm>(rust: &Self, jvm: &mut duchess::Jvm<'jvm>) -> duchess::Result<'jvm, Option<duchess::Local<'jvm, #root_class_name>>> {
                    Ok(Some(duchess::JvmOp::execute_with(rust, jvm)?))
                }
            }
        ))
    }

    fn to_rust_variants(&self) -> Result<Vec<ToRustVariant>, syn::Error> {
        self.input
            .variants()
            .iter()
            .map(|variant| {
                let selector =
                    self.find_method_selector(variant.ast().ident.span(), variant.ast().attrs)?;
                // We're not constructing Java objects in ToRust, so we just need the class name
                // and shouldn't error if the class has multiple constructors that need
                // disambiguation!
                let class = self
                    .reflector
                    .reflect(&selector.class_name(), selector.class_span())?;
                Ok::<_, syn::Error>(ToRustVariant {
                    variant,
                    selector,
                    class,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn check_generics(&self) -> Result<(), syn::Error> {
        if self.input.ast().generics.params.is_empty() {
            Ok(())
        } else {
            // FIXME
            Err(syn::Error::new(
                self.span(),
                "generic structs not yet supported",
            ))
        }
    }

    /// Generates the code to create this variant as part of a `ToRust` impl.
    /// Assumes `self` is the java type and `jvm` is in scope.
    fn variant_to_rust(
        &self,
        obj: TokenStream,
        variant: &VariantInfo,
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        // For each field, construct an expression we will use to initialize its value.
        let mut initializers = VecDeque::new();
        for field in variant.ast().fields {
            if let Some(name) = &field.ident {
                if name == "this" {
                    // Special case for fields named this
                    initializers
                        .push_back(quote_spanned!(name.span() => #obj.global().execute_with(jvm)?));
                } else if self.is_option(&field.ty) {
                    initializers.push_back(quote_spanned!(name.span() =>
                    #obj
                        .#name()
                        .to_rust()
                        .execute_with(jvm)?
                    ));
                } else {
                    initializers.push_back(quote_spanned!(name.span() =>
                    #obj
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
        &self,
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
                    #binding .jderef().upcast().execute_with(jvm)
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

        let args = signature.forbid_capture(|signature| {
            variant
                .bindings()
                .iter()
                .zip(method_arguments.iter())
                .map(|(binding, t)| {
                    Ok::<_, syn::Error>(match t {
                        // deref scalar inputs to bare value
                        Type::Scalar(_) => quote_spanned!(binding.span()=> *#binding),
                        Type::Ref(_) | Type::Repeat(_) => {
                            let java_ty = signature.java_ty(t)?;
                            quote_spanned!(binding.span()=> duchess::ToJava::to_java::<#java_ty>(#binding))
                        }
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })?;

        let class_name = reflected_method
            .class()
            .name
            .to_module_name(method_selector.class_span());
        let method_name = reflected_method
            .name()
            .to_snake_case()
            .to_ident(method_selector.span());

        let pattern = variant.pat();
        Ok(quote_spanned!(self.span() =>
            #pattern => {
                #class_name :: #method_name ( #(#args),* ) .upcast().execute_with(jvm)
            }
        ))
    }

    fn find_method_selector(
        &self,
        span: Span,
        attrs: &[Attribute],
    ) -> Result<MethodSelector, syn::Error> {
        self.find_java_attr(span, attrs)
    }

    /// For enum cases, the root `#[java()]` can't be a method selector, only a class name, so we remain flexible here
    /// in how we try to parse the attr.
    fn find_java_attr<T: Parse>(&self, span: Span, attrs: &[Attribute]) -> Result<T, syn::Error> {
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

fn check_all_extend_root<'a>(
    root: &ClassInfo,
    variants: impl IntoIterator<Item = &'a MethodSelector>,
    upcasts: &Upcasts,
) -> Result<(), syn::Error> {
    if let Some(child) = variants.into_iter().find(|c| {
        !upcasts
            .upcasts_for_generated_class(&c.class_name())
            .contains(&root.this_ref())
    }) {
        Err(syn::Error::new(
            child.class_span(),
            format!("enum variant must extend the root `{}`", root.name),
        ))
    } else {
        Ok(())
    }
}

/// Return the variants sorted topologically by their class heirarchy. Roughly, this means leaf classes must come before
/// any class they extend or implement ("most specific first").
fn order_by_specificity<'a, 'i>(
    variant_classes: &'a BTreeMap<ClassRef, &'a ToRustVariant<'i>>,
    upcasts: &'a Upcasts,
) -> Vec<&'a ToRustVariant<'i>> {
    let class_set = variant_classes.keys().cloned().collect::<BTreeSet<_>>();
    let included_upcasts = variant_classes
        .keys()
        .map(|c| {
            (
                c,
                upcasts
                    .upcasts_for_generated_class(&c.name)
                    .intersection(&class_set)
                    .filter(|&u| u != c)
                    .collect(),
            )
        })
        .collect::<BTreeMap<_, BTreeSet<_>>>();

    // XX: This is an O(N^2) impl that will slow down large enum derives. If we expose an upcasts.direct() fn we can
    // impl a more efficient sort. For now, we loop over the remaining classes, removing any that are upcasts of any
    // others. What're left are the leaf classes. Rinse 'n repeat.
    let mut ordered = Vec::with_capacity(variant_classes.len());
    let mut remaining = class_set.clone();
    while !remaining.is_empty() {
        let mut leaves = remaining.clone();
        for class in &remaining {
            for upcast in &included_upcasts[class] {
                leaves.remove(upcast);
            }
        }
        assert!(!leaves.is_empty());

        ordered.extend(leaves.iter().map(|l| variant_classes[l]));
        for leaf in leaves {
            remaining.remove(&leaf);
        }
    }

    ordered
}

fn unique_variant_classes<'a, 'i>(
    variants: &'a [ToRustVariant<'i>],
) -> Result<BTreeMap<ClassRef, &'a ToRustVariant<'i>>, syn::Error> {
    let mut classes = BTreeMap::new();
    for variant in variants.iter() {
        if classes.insert(variant.class.this_ref(), variant).is_some() {
            return Err(syn::Error::new(
                variant.selector.span(),
                format!(
                    "multiple enum variants for same java class `{}",
                    variant.class.name
                ),
            ));
        }
    }
    Ok(classes)
}

struct ToRustVariant<'i> {
    variant: &'i VariantInfo<'i>,
    selector: MethodSelector,
    class: Arc<ClassInfo>,
}
