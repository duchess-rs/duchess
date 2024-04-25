# Threat model

![Status: Experimental](https://img.shields.io/badge/Status-WIP-yellow)

This page analyzes Duchess's use of the JNI APIs to explain how it guarantees memory safety. Sections:

* [Assumptions](#assumptions) -- covers requirements for safe usage of Duchess which Duchess itself cannot enforce.
* [Code invariants](#code-invariants) -- covers invariants that Duchess maintains
* [Threat vectors that cause UB](#threat-vectors-that-cause-ub) -- ways to create undefined behavior using JNI, and how duchess prevents them (references code invariants and assumptions)
* [Threat vectors that do not cause UB](#threat-vectors-that-do-not-cause-ub) -- suboptimal uses of the JNI that do not create UB; duchess prevents some of these but not all (references code invariants and assumptions)

## Assumptions

We assume three things

1. The Java `.class` files that are present at build time have the same type signatures and public interfaces as the class files that will be present at runtime.
2. The user does not attempt to start the JVM via some other crate in parallel with using duchess methods
    * there can only be one JVM per process. Duchess execute methods will start the JVM if it has not already been started by other means. Duchess methods are internally synchronized, so they can run in parallel, but if a duchess method executes in parallel with code from another library (including another major version of duchess) that attempts to start the JVM, a crash can occur. We recommend starting the JVM explicitly in `main` via `Jvm::builder()`, which will avoid any possibility of encountering this issue.
3. the user does not use the JNI [`PushLocalFrame`][] method to introduce "local variable frames" within the context of `Jvm::with` call
    * duchess not expose [`PushLocalFrame`][], but it is possible to invoke this method via unsafe code or from other crates (e.g., the [`jni` crate's `push_local_frame` method](https://docs.rs/jni/latest/jni/struct.JNIEnv.html#method.push_local_frame)). This method will cause local variables created within its dynamic scope to be released when [`PopLocalFrame`][] is invoked. The `'jvm` lifetime mechanism used to ensure local variables do not escape their scope could be invalidated by these methods. See [the section on the jvm lifetime](#the-jvm-lifetime-mut-jvmjvm-is-the-innermost-scope-for-local-variables) for more details.

[`PushLocalFrame`]: (https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#PushLocalFrame)
[`PopLocalFrame`]: (https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#PopLocalFrame)

## Code invariants

This section introduces invariants maintained by Duchess using Rust's type system as well as careful API design.

### Possessing a `&mut Jvm<'jvm>` implies attached thread

`Jvm` references are obtained with `Jvm::with`. This codepath guarantees

* The JVM has been started (see the [assumptions](#assumptions)), using default settings if needed
* The current thread is attached
    * We maintain a thread-local-variable tracking the current thread status.
    * If the thread is recorded as attached, nothing happens.
    * Otherwise the JNI method to attach the current thread [`AttachCurrentThread`][] is invoked; in that case, the thread will be detached once `with` returns.
    * Users can also invoke `Jvm::attach_thread_permanently` to avoid the overhead of attaching/detaching, which simply sets the thread-local variable to a permanent state and avoids detaching.

[`AttachCurrentThread`]: https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/invocation.html#AttachCurrentThread

### The `'jvm` lifetime `&mut Jvm<'jvm>` is the innermost scope for local variables

References to Java objects of type `J` are stored in a `Local<'jvm, J>` holder. Local references can come from the arguments to native functions or from `JvmOp::execute_with` calls. `execute_with` calls use the `'jvm`' lifetime found on the `Jvm<'jvm>` argument. This allows the `Local` to be used freely within that scope. It is therefore important that `'jvm` be constrained to the **innermost** valid scope.

Inductive argument that this invariant is maintained:

* Base case -- users can only obtain a `Jvm<'jvm>` value via `Jvm::with`, which takes a closure argument of type `for<'jvm> impl FnMut(&mut Jvm<'jvm>)`. Therefore, this closure cannot assume that `'jvm` will outlive the closure call and all local values cannot escape the closure body.
* Inductive case -- all operations performed within a `Jvm::with` maintain the invariant. Violating the invariant would require introducing a new JNI local frame, which can happen in two ways:
    * invoking [`PushLocalFrame`][]: duchess does not expose this operation, and we [assume users do not do this](#assumptions) via some other crate
    * calling into Java code which in turn calls back into Rust code via a `native` method: In this case, we would have a stack with Rust code `R1`, then Java code `J`, then a Rust function `R2` that implements a Java native method. `R1` must have invoked `Jvm::with` to obtain a `&mut Jvm<'jvm>`. If `R1` could somehow give this `Jvm<'jvm>` value to `R2`, `R2` could create locals that would outlive its dynamic extent, violating the invariant. However, `R1` to invoke Java code `J`, `R1` had to invoke a duchess method with `&mut Jvm<'jvm>` as argument, which means that it has given the  Java code unique access to the (unique) `Jvm<'jvm>` value, leant out its only reference, and the Java code does not give this value to `R2`.

**Flaw:**

It is theoretically possible to do something like this...

* `Jvm::with(|jvm1| ...)`
    * stash the `jvm1` somewhere in thread-local data using unsafe code
    * `Jvm::with(|jvm2| ...)`
        * invoke jvm code that calls back into Rust
            * from inside that call, recover the `Jvm<'jvm1>`, alocate a new `Local` with it, and store the result back (unsafely)
    * recover the pair of `jvm1` and the object that was created

...it is difficult to write the code that would do this and it requires unsafe code, but that unsafe code doesn't seem to be doing anything that should not *theoretically* work. Avoiding this is difficult, but if we focus on `execute`, we can make it so that users never directly get their ands on a `Jvm` and make this safe.

### All references to `impl JavaObject` types are JNI local or global references

The [`JavaObject`][] trait is an `unsafe` trait. When implemented on a struct `S`, it means that every `&S` reference must be a JNI local or global references. This trait is implemented for all the structs that duchess creates to represent Java types, e.g., [`duchess::java::lang::Object`][]. This invariant is enforced by the following pattern:

* Each such struct has a private field of type [`Infallible`][], ensuring it could never be constructed via safe code.
* To "construct" an instance of this struct you would use a constructor like [`Object::new`][] which returns an [impl `JavaConstructor`][]; when evaluated it will yield a [`Local`][] wrapper. Locals are only constructed for pointers we get from JNI. Global can be created from Locals (and hence come from JNI too).

[`Infallible`]: https://doc.rust-lang.org/std/convert/enum.Infallible.html
[`JavaObject`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/trait.JavaObject.html
[`duchess::java::lang::Object`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/java/lang/struct.Object.html
[`Object::new`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/java/lang/struct.Object.html#method.new
[impl `JavaConstructor`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/prelude/trait.JavaConstructor.html
[`Local`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/struct.Local.html

### 1:1 correspondence between JNI global/local references and `Global`/`Local`

Every time we create a [`Global`][] value (resp. `Local`), it is created with a new global or local reference on the JNI side as well. The `Drop` for `Global` releases the global (resp., local) reference.

[`Global`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/struct.Global.html

## Threat vectors that cause UB

What follows is a list of specific threat vectors identified by based on the documentation [JNI documentation](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/jniTOC.html) as well as a [checklist of common JNI failures found on IBM documentation](https://www.ibm.com/docs/en/sdk-java-technology/8?topic=jni-checklist).

### When you update a Java object in native code, ensure synchronization of access.

**Outcome of nonadherence:** Memory corruption

**How Duchess avoids this:** We do not support updating objects in native code.

### Cached method and field IDs

From the [JNI documentation](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#accessing_fields_and_methods):

> A field or method ID does not prevent the VM from unloading the class from which the ID has been derived. After the class is unloaded, the method or field ID becomes invalid. The native code, therefore, must make sure to:
>
> * keep a live reference to the underlying class, or
> * recompute the method or field ID
>
> if it intends to use a method or field ID for an extended period of time.

Duchess caches method and field IDs in various places. In all cases, the id is derived from a `Class` reference obtained by invoking [`JavaObject::class`][]. The [`JavaObject::class`][] method is defined to permanently (for the lifetime of the process) cache a global reference to the class object, fulfilling the first criteria ("keep a live reference to the underlying class").

[`JavaObject::class`]: https://duchess-rs.github.io/duchess/rustdoc/doc/duchess/trait.JavaObject.html#tymethod.class

### Local references are tied to the lifetime of a JNI method call

The [JNI manual documents](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#referencing_java_objects) that local references are "valid for the duration of a native method call. Once the method returns, these references will be automatically out of scope." In Duchess, each newly created local reference is assigned to a `Local<'jvm, T>`. This type carries a lifetime (`'jvm`) that derives from the `duchess::Jvm<'jvm>` argument provided to the `JvmOp::execute_with` method. Therefore, the local cannot escape the `'jvm` lifetime on the `Jvm<'jvm>` value; duchess [maintains an invariant that `'jvm` is the innermost JNI local scope](#the-jvm-lifetime-mut-jvmjvm-is-the-innermost-scope-for-local-variables).

### Local references cannot be saved in global variables.

**Outcome of nonadherence:** Random crashes

**How Duchess avoids this:** See discussion [here](#local-references-are-tied-to-the-lifetime-of-a-jni-method-call) and the [`jvm` invariant](#the-jvm-lifetime-mut-jvmjvm-is-the-innermost-scope-for-local-variables).

### Always check for exceptions (or return codes) on return from a JNI function. Always handle a deferred exception immediately you detect it.

**Outcome of nonadherence:** Unexplained exceptions or undefined behavior, crashes

**How Duchess avoids this:** End-users do not directly invoke JNI functions. Within Duchess, virtually all calls to JNI functions use the `EnvPtr::invoke` helper function which checks for exceptions. A small number use `invoke_unchecked`:

* `array.rs`
    * invokes `invoke_unchecked` on [`GetArrayLength`](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetArrayLength), which is not documented as having failure conditions
    * invokes [primitive setter](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Set_PrimitiveType_ArrayRegion_routines) with known-valid bounds
    * invokes [primitive getter](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines) with known-valid bounds
* `cast.rs`
    * invokes infallible method [`IsInstanceOf`](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#IsInstanceOf)
* `find.rs`
    * invokes `GetMethodID` and `GetStaticMethodID` "unchecked" but checks the return value for null and handles exception that occurs
* `raw.rs`
    * invokes `invoke_unchecked` in the implementation of `invoke` :)
* `ref_.rs`
    * invokes `NewLocalRef` with a known-non-null argument
    * invokes `NewLocalRef` with a known-non-null argument
* `str.rs`
    * invokes [`GetStringLength`](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetStringLength) — infallible
    * invokes [`GetStringUTFLength`](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetStringUTFLength) — infallible
    * invokes [`GetStringUTFRegion`](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetStringUTFRegion) with known-valid bounds

### Clear exceptions before invoking other JNI calls

> After an exception has been raised, the native code must first clear the exception before making other JNI calls. 

[Citation.](https://docs.oracle.com/en/java/javase/17/docs/specs/jni/design.html#exception-handling)

**Outcome of nonadherence:** Undefined behavior.

**How Duchess avoids this:** When we detect an exception, we always clear the exception immediately before returning a `Result`.

### Illegal argument types

[JNI document states](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#reporting_programming_errors):

> Reporting Programming Errors
>
> The JNI does not check for programming errors such as passing in NULL pointers or illegal argument types.
>
> The programmer must not pass illegal pointers or arguments of the wrong type to JNI functions. Doing so could result in arbitrary consequences, including a corrupted system state or VM crash.

**How Duchess avoids this:** We generate strongly typed interfaces based on the signatures found in the class files and we [assume that the same class files are present at runtime](#assumptions).

**Example tests:**

* `type_mismatch_*.rs` in the test directory

## Threat vectors that do not cause UB

### Invoke execution occurred regularly

[Recommendation:](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#asynchronous_exceptions)

> Native methods should insert `ExceptionOccurred()` checks in necessary places (such as in a tight loop without other exception checks) to ensure that the current thread responds to asynchronous exceptions in a reasonable amount of time.

**Outcome of nonadherence:** Asynchronous exceptions won't be detected.

**How Duchess avoids this:** We check this flag at every interaction with the JVM but not other times; it is possible for Rust code to execute for arbitrary amounts of time without checkin the flag. Asynchronous exceptions are not recommended in modern code and the outcome of not checking is not undefined behavior.

### Local variable capacity

Each JNI frame has a guaranteed capacity which can be extended via `EnsureLocalCapacity`. This limit is largely advisory, and exceeding it does not cause UB. The documentation states:

> For backward compatibility, the VM allocates local references beyond the ensured capacity. (As a debugging support, the VM may give the user warnings that too many local references are being created. In the JDK, the programmer can supply the -verbose:jni command line option to turn on these messages.) The VM calls FatalError if no more local references can be created beyond the ensured capacity.

**Outcome of nonadherence:** Slower performance or, in extreme cases, aborting the process via reporting a [Fatal Error](https://docs.oracle.com/en/java/javase/21/vm/error-reporting.html).

**How Duchess avoids this:**

* Duchess is not aware of this limit and does not limit the number of local variables that will be created. If needed, we could support annotations or other means.
* However, if using `Duchess` in its recommended configuration (with `execute` calls), all local variables will be cleaned up in between operations, and operations always create a finite (and statically known) number of locals

### Ensure that every global reference created has a path that deletes that global reference.

**Outcome of nonadherence:** Memory leak

**How Duchess avoids this:** Because there is a [1:1 correspondence between JNI global references](#11-correspondence-between-jni-global-references-and-global)

Every time we create a global reference, we store it in a `Global` type. The destructor on this type will free the reference.

### Memory exhaustion from too many local references

[JNI reference states:](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#global_and_local_references)

> However, there are times when the programmer should explicitly free a local reference. Consider, for example, the following situations:
>
> * A native method accesses a large Java object, thereby creating a local reference to the Java object. The native method then performs additional computation before returning to the caller. The local reference to the large Java object will prevent the object from being garbage collected, even if the object is no longer used in the remainder of the computation.
> * A native method creates a large number of local references, although not all of them are used at the same time. Since the VM needs a certain amount of space to keep track of a local reference, creating too many local references may cause the system to run out of memory. For example, a native method loops through a large array of objects, retrieves the elements as local references, and operates on one element at each iteration. After each iteration, the programmer no longer needs the local reference to the array element.
>
> The JNI allows the programmer to manually delete local references at any point within a native method.

**Outcome of nonadherence:** Memory exhaustion.

**How Duchess avoids this:** We do not expect users to do fine-grained interaction with Java objects in this fashion and we do not provide absolute protection from memory exhaustion. However, we do mitigate the likelihood, as the `Local` type has a destructor that deletes local references. Therefore common usage patterns where a `Local` is created and then dropped within a loop (but not live across loop iterations) would result in intermediate locals being deleted.

### Native references crossing threads

[JNI document states:](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#global_and_local_references)

> Local references are only valid in the thread in which they are created. The native code must not pass local references from one thread to another.

**Outcome of nonadherence:** Memory exhaustion.

**How Duchess avoids this:** Duchess does not prevent this, but the result is not UB, and we do not expect users to do fine-grained interaction with Java objects in this fashion.

### Ensure that you use the isCopy and mode flags correctly. See Copying and pinning.

**Outcome of nonadherence:** Memory leaks and/or heap fragmentation

**How Duchess avoids this:** Duchess does not currently make use of the methods to gain direct access to Java array contents, so this is not relevant.

### Ensure that array and string elements are always freed.

**Outcome of nonadherence:** Memory leak

**How Duchess avoids this:** Unclear what this exactly means, to be honest, but we make no special effort to prevent it. However, memory leaks are largely unlikely in Duchess due to having a destructor on `Global`.
