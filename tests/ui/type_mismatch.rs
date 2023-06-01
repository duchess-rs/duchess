//@compile-flags: --crate-type lib

duchess::java_package! {
    package java.lang;

    public class java.lang.Object { //~ ERROR: generic type parameter `Id { data: "bool" }` not among in-scope parameters: []
        public java.lang.Object();
        public native bool hashCode();
    }
}

fn main() {}
