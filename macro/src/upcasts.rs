use std::collections::{BTreeMap, BTreeSet};

use crate::{
    class_info::{ClassInfo, ClassRef, DotId, Id},
    substitution::{Substitute, Substitution},
};

/// A map storing the transitive upcasts for each class that we are generated (and potentially additional classes).
///
/// There is one caveat: we only compute the transitive superclasses based on the classes that are input to
/// the proc macro. The problem is that we can only inspect the tokens presented to us. While we could reflect
/// on the Java classes directly, we don't know what subset of the supertypes the user has chosen to reflect into
/// Rust. Therefore, we stop our transitive upcasts at the "water's edge" -- i.e., at the point where we
/// encounter classes that are outside our package.
#[derive(Default, Debug)]
pub struct Upcasts {
    map: BTreeMap<DotId, ClassUpcasts>,
}

#[derive(Debug)]
pub struct ClassUpcasts {
    generics: Vec<Id>,
    extends: BTreeSet<ClassRef>,
}

impl<'a> FromIterator<&'a ClassInfo> for Upcasts {
    fn from_iter<T: IntoIterator<Item = &'a ClassInfo>>(iter: T) -> Self {
        let mut upcasts = Upcasts::default();

        for class_info in iter {
            upcasts.insert_direct_upcasts(class_info);
        }

        upcasts.insert_hardcoded_upcasts();

        upcasts.compute_transitive_upcasts();

        upcasts
    }
}

impl Upcasts {
    /// Returns the transitive superclasses / interfaces of `name`.
    /// These will reference generic parameters from in the class declaration of `name`.
    pub fn upcasts_for_generated_class(&self, name: &DotId) -> &BTreeSet<ClassRef> {
        &self.map[name].extends
    }

    /// Insert the direct (declared by user) superclasses of `class` into the map.
    fn insert_direct_upcasts(&mut self, class: &ClassInfo) {
        let mut upcasts = ClassUpcasts {
            generics: class.generics.iter().map(|g| g.id.clone()).collect(),
            extends: BTreeSet::default(),
        };

        upcasts.extends.insert(class.this_ref());

        for c in class.extends.iter().chain(&class.implements) {
            upcasts.extends.insert(c.clone());
        }

        upcasts.extends.insert(ClassRef {
            name: DotId::object(),
            generics: vec![],
        });

        let old_value = self.map.insert(class.name.clone(), upcasts);
        assert!(old_value.is_none());
    }

    fn insert_hardcoded_upcasts(&mut self) {
        let mut insert = |c: DotId, d: DotId| {
            self.map
                .entry(c)
                .or_insert_with(|| ClassUpcasts {
                    generics: vec![],
                    extends: BTreeSet::default(),
                })
                .extends
                .insert(ClassRef {
                    name: d,
                    generics: vec![],
                });
        };

        // N.B. we don't insert the reflexive `C extends C` relations here, which means we don't
        // necessarily have 100% parity between these hardcoded types and types created by
        // `insert_direct_upcasts`. This is a micro-optimization that I can't resist.
        // The idea is that reflexive impls aren't necessary because they only matter when
        // we are generating `Upcast` impls, and we only generate `Upcast` impls for those
        // classes that appear in our package declarations.

        insert(DotId::runtime_exception(), DotId::exception());
        insert(DotId::exception(), DotId::throwable());
        insert(DotId::throwable(), DotId::object());
    }

    /// Extend the map with transitive upcasts for each of its entries. i.e., if class `A` extends `B`,
    /// and `B` extends `C`, then `A` extends `C`.
    fn compute_transitive_upcasts(&mut self) {
        let class_names: Vec<DotId> = self.map.keys().cloned().collect();
        loop {
            let mut changed = false;

            for n in &class_names {
                // Extend by one step: for each class `c` extended by `n`,
                // find superclasses of `c`.
                let indirect_upcasts: Vec<ClassRef> = self.map[n]
                    .extends
                    .iter()
                    .flat_map(|c| self.upcasts(c))
                    .collect();

                // Insert those into the set of superclasses for `n`.
                // If the set changed size, then we added a new entry,
                // so we have to iterate again.
                let c_u = self.map.get_mut(n).unwrap();
                let len_before = c_u.extends.len();
                c_u.extends.extend(indirect_upcasts);
                changed |= c_u.extends.len() != len_before;
            }

            if !changed {
                break;
            }
        }
    }

    /// Find the upcasts for `class_ref`: look up the current map entry for
    /// the given class and substitute the given values for its generic parameters.
    fn upcasts(&self, class_ref: &ClassRef) -> Vec<ClassRef> {
        let Some(c_u) = self.map.get(&class_ref.name) else {
            // Upcasts to classes outside our translation unit:
            // no visibility, just return empty vector.
            return vec![];
        };

        assert_eq!(class_ref.generics.len(), c_u.generics.len());

        let subst: Substitution<'_> = c_u.generics.iter().zip(&class_ref.generics).collect();

        c_u.extends.iter().map(|c| c.substitute(&subst)).collect()
    }
}
