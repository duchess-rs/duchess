use crate::{
    argument::DuchessDeclaration,
    class_info::{
        ClassInfo, ClassRef, Constructor, Id, RefType, ScalarType, SpannedClassInfo, Type,
    },
    span_error::SpanError,
};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote_spanned;

impl DuchessDeclaration {
    pub fn into_tokens(mut self) -> Result<TokenStream, SpanError> {
        todo!()
    }
}

impl SpannedClassInfo {
    pub fn into_tokens(mut self) -> TokenStream {
        let struct_name = self.struct_name();
        let cached_class = self.cached_class();
        let constructors: Vec<_> = self
            .info
            .constructors
            .iter()
            .map(|c| self.constructor(c))
            .collect();

        quote_spanned! {
            self.span =>

            pub struct #struct_name {
                _dummy: ()
            }

            // Hide other generated items
            const _: () = {
                use duchess::{
                    java,
                    plumbing,
                    IntoJava, IntoRust, JavaObject, Jvm, JvmOp, Local,
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


            };
        }
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

    fn constructor(&self, constructor: &Constructor) -> TokenStream {
        let mut sig = Signature::new(self.span, &[]);
        let args: Vec<_> = constructor
            .args
            .iter()
            .map(|ty| sig.java_type(ty))
            .collect();

        todo!()
    }

    fn struct_name(&self) -> Ident {
        Ident::new(&self.info.name, self.span)
    }

    /// Returns a class name with `/`, like `java/lang/Object`.
    fn jni_class_name(&self) -> Literal {
        self.info.name.to_jni_name(self.span)
    }
}

struct Signature {
    span: Span,
    generics: Vec<Ident>,
    where_bounds: Vec<TokenStream>,
}

impl Signature {
    pub fn new(span: Span, generics: &[Id]) -> Self {
        let mut this = Signature {
            span,
            generics: vec![],
            where_bounds: vec![],
        };
        for generic in generics {
            let ident = this.java_type_parameter_ident(generic);
            this.generics.push(ident);
        }
        this
    }

    fn fresh_generic(&mut self) -> Ident {
        let mut i = self.generics.len();
        let ident = Ident::new(&format!("P{}", i), self.span);
        self.generics.push(ident.clone());
        ident
    }

    fn push_where_bound(&mut self, t: TokenStream) {
        self.where_bounds.push(t);
    }

    fn java_type(&mut self, ty: &Type) -> TokenStream {
        match ty {
            Type::Ref(ty) => self.java_ref_ty(ty),

            Type::Scalar(ty) => match ty {
                ScalarType::Int => quote_spanned!(self.span => i32),
                ScalarType::Long => quote_spanned!(self.span => i64),
                ScalarType::Short => quote_spanned!(self.span => i16),
                ScalarType::Byte => quote_spanned!(self.span => i8),
                ScalarType::F64 => quote_spanned!(self.span => f64),
                ScalarType::F32 => quote_spanned!(self.span => f32),
                ScalarType::Boolean => quote_spanned!(self.span => bool),
            },
        }
    }

    fn java_ref_ty(&mut self, ty: &RefType) -> TokenStream {
        match ty {
            RefType::Class(ty) => self.class_ref_ty(ty),
            RefType::Array(e) => {
                let e = self.java_ref_ty(e);
                quote_spanned!(self.span => java::JavaArray<#e>)
            }
            RefType::TypeParameter(t) => {
                let ident = self.java_type_parameter_ident(t);
                quote_spanned!(self.span => #ident)
            }
            RefType::Extends(ty) => {
                let e = self.java_ref_ty(ty);
                let g = self.fresh_generic();
                self.push_where_bound(quote_spanned!(self.span => #g : AsRef<#e>));
                quote_spanned!(self.span => #g)
            }
            RefType::Super(_) => {
                let g = self.fresh_generic();
                // FIXME: missing where bound, really
                quote_spanned!(self.span => #g)
            }
            RefType::Wildcard => {
                let g = self.fresh_generic();
                quote_spanned!(self.span => #g)
            }
        }
    }

    fn class_ref_ty(&mut self, ty: &ClassRef) -> TokenStream {
        let ClassRef { name, generics } = ty;
        let rust_name = name.to_module_name(self.span);
        if generics.len() == 0 {
            quote_spanned!(self.span => #rust_name)
        } else {
            let rust_tys: Vec<_> = generics.iter().map(|t| self.java_ref_ty(t)).collect();
            quote_spanned!(self.span => #rust_name < #(#rust_tys),* >)
        }
    }

    fn java_type_parameter_ident(&self, t: &Id) -> Ident {
        Ident::new(&format!("J{}", t), self.span)
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
