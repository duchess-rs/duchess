# Test crates

This folder contains crates that test the end-to-end behavior of `duchess`. This setup makes it easy to test different ways of setting the CLASSPATH, or usages of real-world libraries whose JAR is too big to be included in the repository.

The ui tests are organized into 2 directories, ui and java_ui
    * ui - Rust tests that test the ability of Rust initiated applications to interop with Java
    * java_ui - Java tests that test the ability of Java initiated applications to interop with Rust

java_ui tests are run after the completion of the Rust ui tests. This allows rust tests to be add in the ui directory which are compiled into *.so files that the java tests consume. For an example of this interaction see [native_fn_callable_from_java.rs](duchess-java-tests/tests/ui/native_fn_callable_from_java.rs) and [JavaCanCallRustJavaFunction.java](duchess-java-tests/tests/java_ui/java_to_rust_greeting/JavaCanCallRustJavaFunction.java)
