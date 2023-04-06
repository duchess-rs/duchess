# The `java_package` macro

The `java_package` macro creates Rust structures to interact with a java package. It uses `javap` to read the class files and find the existing methods and other details so that you don't have to do any manual work.

