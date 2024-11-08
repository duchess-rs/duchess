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

declare_regex!(java_package() = r"(?m)^\s*(duchess|duchess_macro)::java_package! *\{");

#[cfg(test)]
mod test {
    #[test]
    fn test_java_package_regex() {
        assert!(super::java_package().is_match(r#"       duchess_macro::java_package! { "#));
        let java_file = r#"
    NB. in doctests, the current crate is already available as duchess.

    duchess_macro::java_package! {
        package java.lang;

        public class java.lang.Object {
            public java.lang.Object();
            public native int hashCode();
            public boolean equals(java.lang.Object);
            public java.lang.String toString();
            public final native void notify();
            public final native void notifyAll();"#;
        assert!(super::java_package().is_match(java_file));
    }
}
