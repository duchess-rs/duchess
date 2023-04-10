# JVM Operations

*JVM operations* correspond to code that will execute on the JVM. Like futures and iterators, JVM operations are lazy. This means that you compose them together using a series of method calls and, once you've built up the entire thing that you want to do, you invoke the `execute` method, giving it a [`&mut Jvm`](./jvm.md) to execute on. This lazy style is convenient to use, because you only have to supply the `jvm` argument once, but it also gives duchess a chance to optimize for fewer JNI invocations, making your code run faster.

