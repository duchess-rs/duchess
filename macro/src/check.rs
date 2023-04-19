use crate::{
    argument::{MemberListing, MemberListingElement},
    class_info::{
        ClassInfo, ClassName, ClassRef, Constructor, Method, RefType, RootMap, SpannedClassInfo,
        Type,
    },
    span_error::SpanError,
};

impl RootMap {
    pub fn check(&self) -> Result<(), SpanError> {
        let mut errors = vec![];

        for class_name in &self.class_names() {
            let ci = self.find_class(class_name).unwrap();
            ci.check(self, &mut |e| errors.push(e));
        }

        // FIXME: support multiple errors
        if let Some(e) = errors.pop() {
            Err(e)
        } else {
            Ok(())
        }
    }
}

impl SpannedClassInfo {
    fn check(&self, root_map: &RootMap, push_error: &mut impl FnMut(SpanError)) {
        let mut push_error_message = |m: String| {
            push_error(SpanError {
                span: self.span,
                message: format!("error in class `{}`: {m}", self.info.name),
            });
        };

        for cref in &self.info.extends {
            cref.check(root_map, &mut |m| {
                push_error_message(format!("{m}, but is extended by `{}`", self.info.name))
            });
        }

        for cref in &self.info.implements {
            cref.check(root_map, &mut |m| {
                push_error_message(format!("{m}, but is implemented by `{}`", self.info.name));
            });
        }

        for c in self.selected_constructors() {
            c.check(root_map, &mut |m| {
                push_error_message(format!(
                    "{m}, which appears in constructor {}",
                    c.to_method_sig(&self.info)
                ));
            });
        }

        for c in self.selected_methods() {
            c.check(root_map, &mut |m| {
                push_error_message(format!(
                    "{m}, which appears in method {}",
                    c.to_method_sig()
                ));
            });
        }

        // Check that each of the method filters corresponds to an actual method that exists
        self.members.check(root_map, &self.info, push_error);
    }
}

impl ClassRef {
    fn check(&self, root_map: &RootMap, push_error: &mut impl FnMut(String)) {
        let class_name = ClassName::from(&self.name);

        let (package_name, class_id) = class_name.split();
        if let Some(package) = root_map.find_package(package_name) {
            if let None = package.find_class(&class_id) {
                push_error(format!(
                    "class `{class_name}` not in list of classes to be translated"
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
