duchess::java_package! { //~ ERROR: proc macro panicked
    //~^ HELP: generic type parameter `Id { data: "bool" }` not among in-scope parameters: []
    package java.lang;

    public class java.lang.Object {
        public java.lang.Object();
        public native bool hashCode();
    }
}
