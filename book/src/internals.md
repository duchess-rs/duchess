# Internals

How the generated code works and why.

## Representing Java objects

Java objects are represented by a dummy struct:

```rust
pub struct MyObject {
    _dummy: ()
}
```

which implements the `JavaObject` trait:

```rust
unsafe impl JavaObject for MyObject { }
```

### References to java objects

This unsafe impl asserts that **every reference `&MyObject` is actually a `sys::jobject`**. This allows us to create a `sys::jobject` simply by casting the `&MyObject`. We maintain that invariant by never allowing users to own a `MyObject` directly; they can only get various kinds of pointers to `MyObject` types (covered below).

Given a reference `&'l MyObject`, the lifetime `'l` is tied to the JVM's "local frame" length. If this Rust code is being invoked via the JNI, then `'l` is the duration of the outermost JNI call.

**Important:** Our design does not support nested local frames and thus we don't expose those in our API. This simplifying assumption means that we can connect the lifetimes of local variables to one another, rather than having to tie them back to some `jni` context.

### `Local` Java objects

Whenever we invoke a JNI method, or execute a construct, it creates a new local handle. These are returned to the user as a `Local<'jni, MyObject>` struct, where the `'jni` is (again) the lifetime of the local frame. Internally, the `Local` struct is actually just a `jobject` pointer, though we cast it to `*mut MyObject`; it supports deref to `&'jni MyObject` in the natural way. Note that this maintains the representation invariant for `&MyObject` (i.e., it is still a jobject pointer).

`Local` has a `Drop` impl that deletes the local handle. This is important because there is a limit to the number of references you can have in the JNI, so you may have to ensure that you drop locals in a timely fashion.

### `Global` Java objects

The `jdk` object offers a method to create a Global reference a Java object. Global references can outlive the current frame. They are represented by a `Global<MyObject>` type, which is a newtype'd `sys::jobject` as well that represents a global handle. This type has a `Drop` impl which deletes the global reference and supports `Deref` in the same way as `Local`.

### null

The underlying `sys::jobject` can be null, but we maintain the invariant that this is never the case, instead using `Option<&R>` etc.


