use regex::Regex;
use std::sync::OnceLock;

macro_rules! declare_regex {
    ($name:ident() = $regex:expr) => {
        pub(crate) fn $name() -> &'static Regex {
            static STATIC: OnceLock<Regex> = OnceLock::new();
            STATIC.get_or_init(|| Regex::new($regex).unwrap())
        }
    };
}

declare_regex!(impl_java_interface() = r"#\[duchess::impl_java_interface\]");

declare_regex!(java_package() = r"duchess::java_package! *\{");
