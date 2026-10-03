//! Knowledge of the duchess prelude (`src/java.rs`) for validation.
//!
//! A `java.*` reference that is not declared in the current `java_package!`
//! invocation resolves to the prelude (see `JavaPathResolver` in
//! `class_info`), so validation accepts precisely the references codegen can
//! fulfill: the ones named here. Anything else in a locally-declared package
//! (e.g. `java.util.Nope`) is still an error (see `ClassRef::check`).
//!
//! This table hand-mirrors the `java.*` declarations in `src/java.rs`; keep
//! them in sync (the `prelude_table_matches_java_rs` test enforces this). It
//! is mirrored by hand rather than generated because `duchess-reflect` is
//! built and published as an independent crate and cannot read sibling files
//! at build time.

use crate::class_info::{DotId, Id};

/// The classes declared in the duchess prelude (`src/java.rs`), as path
/// segments. See [`is_prelude_class`].
pub(crate) const PRELUDE_CLASSES: &[&[&str]] = &[
    &["java", "lang", "Class"],
    &["java", "lang", "Exception"],
    &["java", "lang", "Long"],
    &["java", "lang", "NullPointerException"],
    &["java", "lang", "Object"],
    &["java", "lang", "Record"],
    &["java", "lang", "RuntimeException"],
    &["java", "lang", "StackTraceElement"],
    &["java", "lang", "String"],
    &["java", "lang", "Throwable"],
    &["java", "lang", "management", "GarbageCollectorMXBean"],
    &["java", "lang", "management", "ManagementFactory"],
    &["java", "lang", "management", "MemoryManagerMXBean"],
    &["java", "lang", "management", "MemoryMXBean"],
    &["java", "lang", "management", "MemoryPoolMXBean"],
    &["java", "lang", "management", "MemoryType"],
    &["java", "lang", "management", "MemoryUsage"],
    &["java", "time", "Instant"],
    &["java", "util", "ArrayList"],
    &["java", "util", "Date"],
    &["java", "util", "HashMap"],
    &["java", "util", "List"],
    &["java", "util", "Map"],
];

/// True if `name` is a class declared in the duchess prelude.
pub(crate) fn is_prelude_class(name: &DotId) -> bool {
    let segments: &[Id] = name;
    PRELUDE_CLASSES.iter().any(|candidate| {
        candidate.len() == segments.len()
            && candidate
                .iter()
                .zip(segments.iter())
                .all(|(candidate, segment)| &segment[..] == *candidate)
    })
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeSet, fs, path::PathBuf};

    use super::*;

    #[test]
    fn is_prelude_class_matches_declared_prelude() {
        for name in [
            "java.lang.Object",
            "java.lang.String",
            "java.util.List",
            "java.util.ArrayList",
            "java.util.Map",
            "java.util.HashMap",
            "java.util.Date",
            "java.time.Instant",
            "java.lang.management.MemoryMXBean",
        ] {
            assert!(is_prelude_class(&DotId::parse(name)), "{name}");
        }
        for name in [
            "java.util.Set",
            "java.util.Nope",
            "java.io.OutputStream",
            "com.foo.Bar",
        ] {
            assert!(!is_prelude_class(&DotId::parse(name)), "{name}");
        }
    }

    /// The `PRELUDE_CLASSES` table must mirror the `java.*` declarations in
    /// `src/java.rs`. Comment lines are ignored: several commented-out members
    /// mention classes (e.g. `java.util.Set`) that are deliberately *not* in
    /// the prelude.
    #[test]
    fn prelude_table_matches_java_rs() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/java.rs");
        let contents = fs::read_to_string(&path).unwrap();

        let mut declared: BTreeSet<String> = BTreeSet::new();
        let mut package: Vec<String> = Vec::new();
        for line in contents.lines() {
            let code = line.split("//").next().unwrap().trim();
            if let Some(name) = code
                .strip_prefix("package ")
                .map(|name| name.trim_end_matches(';').trim())
            {
                package = name.split('.').map(str::to_string).collect();
                continue;
            }
            let tokens: Vec<&str> = code.split_whitespace().collect();
            for (i, token) in tokens.iter().enumerate() {
                if !matches!(*token, "class" | "interface" | "enum" | "record") {
                    continue;
                }
                if let Some(name) = tokens.get(i + 1) {
                    let name = name.split(['<', '{', ';']).next().unwrap();
                    if name.contains('.') {
                        declared.insert(name.to_string());
                    } else {
                        declared.insert(
                            package
                                .iter()
                                .chain(std::iter::once(&name.to_string()))
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("."),
                        );
                    }
                    break;
                }
            }
        }

        let table: BTreeSet<String> = PRELUDE_CLASSES
            .iter()
            .map(|segments| segments.join("."))
            .collect();
        assert_eq!(table, declared);
    }
}
