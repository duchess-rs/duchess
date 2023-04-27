# Internals

How the generated code works and why.

## Tracking the JNI environment



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

## Exceptions 

The [JNI exposes Java exception state](https://docs.oracle.com/javase/7/docs/technotes/guides/jni/spec/functions.html#wp5234) via 
 * `ExceptionCheck()` returning `true` if an unhandled exception has been thrown
 * `ExceptionOccurred()` returning a local reference to the thrown object
 * `ExceptionClear()` clearing the exception (if any)

If an exception has occurred and isn't cleared before the next JNI call, the invoked Java code will immediately "see" the exception. Since this can cause an exception to propagate outside of the normal stack bubble-up, the `jni` crate always calls `ExceptionCheck` after all JNI calls and returns `Err(jni::errors::Error::JavaException)` if one has occurred. 

The `JavaException` value isn't useful on its own since it doesn't contain a reference to the thrown object. Also, if we were to expose it directly to users who may ignore the `Err` case of a Duchess result, the exception would be left uncleared and poison the next JNI call. We therefore wrap all `jni` calls with `duchess::plumbing::with_jni_env()` to check for the `JavaException` error, materialize the thrown object into `duchess::Error::Thrown(obj)`, and clear the exception state. 

## `jni` crate conventions

While we may unwind our dendency on the `jni` crate wrappers over the JNI, for now we have the following conventions for using `jni` functions:
 1. All `jni` functions that return a `Result` must be called inside of `duchess::plumbing::with_jni_env()` to properly materialize and clear exception state between JNI calls. See [Exceptions](#exceptions).

## Frequently asked questions

Covers various bits of rationale.

### Why do you not supported nested frames in the JNI?

We do not want users to have to supply a context object on every method call, so instead we take the lifetime of the returned java reference and tie it to the inputs:

```rust
// from Java, and ignoring exceptions / null for clarity:
//
// class MyObject { ReturnType some_method(); }
impl MyObject {
    pub fn some_method<'jvm>(&'jvm self) -> Local<'jvm, ReturnType> {
        //                    ----                ----
        //           Lifetime in the return is derived from `self`.
        ...
    }
}
```

This implies though that every 

We have a conflict:

* Either we make every method take a jdk pointer context.
* Or... we go into a suspended mode...

```rust
MyObject::new(x, y, z)
    .execute(jdk);

MyObject::new(x, y, z)
    .blah(something)
    .blah(somethingElse)
    .execute(jdk);

MyObject::new(x, y, z)
    .blah(something)
    .blah(somethingElse)
    .map(|x| {
        x.someMethod()
    })
    .execute(jdk);
```

...this can start by compiling to jdk calls... and then later we can generate byte code and a custom class, no?



If we supported nested frames, we would have to always take a "context" object and use that to derive the lifetime of each `Local<'l, MyObject>` reference. But that is annoying for users, who then have to add an artificial seeming environment as a parameter to various operations. (As it is, we still need it for static methods and constructors, which is unfortunate.)

