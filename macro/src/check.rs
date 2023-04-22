use crate::{
    argument::{JavaClass, MemberListing, MemberListingElement},
    class_info::{ClassInfo, ClassRef, Constructor, DotId, Method, RefType, RootMap, Type},
    reflect::Reflector,
    span_error::SpanError,
};

impl RootMap {
    pub fn check(&self, reflector: &mut Reflector) -> Result<(), SpanError> {
        let mut errors = vec![];

        for class_name in &self.class_names() {
            let ci = self.find_class(class_name).unwrap();
            ci.check(class_name, self, reflector, &mut |e| errors.push(e))?;
        }

        // FIXME: support multiple errors
        if let Some(e) = errors.pop() {
            Err(e)
        } else {
            Ok(())
        }
    }
}

impl JavaClass {
    fn check(
        &self,
        class_name: &DotId,
        root_map: &RootMap,
        reflector: &mut Reflector,
        push_error: &mut impl FnMut(SpanError),
    ) -> Result<(), SpanError> {
        let info = reflector.reflect_at(class_name, self.class_span)?;

        let mut push_error_message = |m: String| {
            push_error(SpanError {
                span: self.class_span,
                message: format!("error in class `{}`: {m}", self.class_name),
            });
        };

        for cref in &info.extends {
            cref.check(root_map, &mut |m| {
                push_error_message(format!("{m}, but is extended by `{}`", self.class_name))
            });
        }

        for cref in &info.implements {
            cref.check(root_map, &mut |m| {
                push_error_message(format!("{m}, but is implemented by `{}`", self.class_name));
            });
        }

        for c in info.selected_constructors(&self.members) {
            c.check(root_map, &mut |m| {
                push_error_message(format!(
                    "{m}, which appears in constructor {}",
                    c.to_method_sig(info)
                ));
            });
        }

        for c in info.selected_methods(&self.members) {
            c.check(root_map, &mut |m| {
                push_error_message(format!(
                    "{m}, which appears in method {}",
                    c.to_method_sig()
                ));
            });
        }

        // Check that each of the method filters corresponds to an actual method that exists
        self.members.check(root_map, info, push_error);

        Ok(())
    }
}

impl ClassRef {
    fn check(&self, root_map: &RootMap, push_error: &mut impl FnMut(String)) {
        let (package_name, class_id) = self.name.split();
        if let Some(package) = root_map.find_package(package_name) {
            if let None = package.find_class(&class_id) {
                push_error(format!(
                    "class `{}` not in list of classes to be translated",
                    self.name,
                ))
            }
        }
    }
}

impl MemberListing {
    fn check(&self, root_map: &RootMap, class: &ClassInfo, push_error: &mut impl FnMut(SpanError)) {
        for element in &self.elements {
            element.check(root_map, class, push_error);
        }
    }
}

impl MemberListingElement {
    fn check(&self, root_map: &RootMap, class: &ClassInfo, push_error: &mut impl FnMut(SpanError)) {
        match self {
            MemberListingElement::Wildcard(ml) => ml.check(root_map, class, push_error),
            MemberListingElement::Named(sm) => {
                if !class
                    .constructors
                    .iter()
                    .any(|ctor| sm.method_sig.matches_constructor(class, ctor))
                    && !class.methods.iter().any(|m| sm.method_sig.matches(m))
                {
                    push_error(SpanError {
                        span: sm.span,
                        message: format!("no member of `{}` matches this signature", class.name),
                    })
                }
            }
        }
    }
}

impl Constructor {
    fn check(&self, root_map: &RootMap, mut push_error: impl FnMut(String)) {
        for ty in &self.argument_tys {
            ty.check(root_map, &mut push_error);
        }
    }
}

impl Method {
    fn check(&self, root_map: &RootMap, mut push_error: impl FnMut(String)) {
        for ty in &self.argument_tys {
            ty.check(root_map, &mut push_error);
        }
    }
}

impl Type {
    fn check(&self, root_map: &RootMap, push_error: &mut impl FnMut(String)) {
        match self {
            Type::Ref(r) => r.check(root_map, push_error),
            Type::Scalar(_) => (),
            Type::Repeat(ty) => ty.check(root_map, push_error),
        }
    }
}

impl RefType {
    fn check(&self, root_map: &RootMap, push_error: &mut impl FnMut(String)) {
        match self {
            RefType::Class(cref) => cref.check(root_map, push_error),
            RefType::Array(array) => array.check(root_map, push_error),
            RefType::TypeParameter(_) => (),
            RefType::Extends(ty) => ty.check(root_map, push_error),
            RefType::Super(ty) => ty.check(root_map, push_error),
            RefType::Wildcard => (),
        }
    }
}
