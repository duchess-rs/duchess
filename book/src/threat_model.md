# Threat model

This page analyzes Duchess's use of the JNI APIs to explain how it guarantees memory safety in each case. 

## Code invariants

This section introduces invariants maintained by Duchess using Rust's type system as well as careful API design.

### All references to `impl JavaObject` types are JNI local or global references

The [`JavaObject`]

### 1:1 correspondence between JNI global references and `Global`

## Specific threat vectors

What follows is a list of specific threat vectors identified by based on the documentation [JNI documentation](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/jniTOC.html) as well as a [checklist of common JNI failures found on IBM documentation](https://www.ibm.com/docs/en/sdk-java-technology/8?topic=jni-checklist).

### When you update a Java object in native code, ensure synchronization of access.

**Outcome of nonadherence:** Memory corruption

**How Duchess avoids this:** We do not support updating objects in native code.


### Ensure that every global reference created has a path that deletes that global reference.

**Outcome of nonadherence:** Memory leak

**How Duchess avoids this:** Every time we create a global reference, we store it in a `Global` type. The destructor on this type will free the reference.



---

## Attaching and detaching from threads

To use the JNI one must obtain a `JNIEnv*` pointer.
This pointer is specific to a particular OS thread and cannot be used from other threads.
You can obtain the "first" `JNIEnv*` pointer in two ways

* By explicitly attaching a Rust thread to the JVM using [`AttachCurrentThread`];
* As a parameter that is given to a Rust function when it is being used to implement a Java `native` function.


This can be done The JNI can only be used on particular threads 
To be used within a particular thread, the JNI must be "attached" to that thread.
Attaching gives access to a `JNIEnv*` pointer that is specific to the current thread.

[`AttachCurrentThread`]: https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/invocation.html#AttachCurrentThread

## References to Java objects

As [documented in the JNI manual](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#referencing_java_objects),
native can reference Java objects in one of two ways:

* Local references, which are valid for the duration of a native method call. Once the method returns, these references will be automatically out of scope.
* Global references, which remain valid until that are explicitly freed.

In both cases, these are not direct references to the heap, but rather a pointer to internal JVM storage which stores the real reference.
This permits the JVM to compact and relocate Java objects even when native code is executing.

In Duchess, local references are always represented using a `Local<'jvm, T>` type, where `'jvm` represents the scope of the current JVM invocation.

## Cached method and field IDs

From the [JNI documentation](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#accessing_fields_and_methods):

> A field or method ID does not prevent the VM from unloading the class from which the ID has been derived. After the class is unloaded, the method or field ID becomes invalid. The native code, therefore, must make sure to:
>
> * keep a live reference to the underlying class, or
> * recompute the method or field ID
>
> if it intends to use a method or field ID for an extended period of time.


## References to Java objects

### Invoke execution occurred regularly

[Recommendation:](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#asynchronous_exceptions)

> Native methods should insert ExceptionOccurred()checks in necessary places (such as in a tight loop without other exception checks) to ensure that the current thread responds to asynchronous exceptions in a reasonable amount of time.

**Outcome of nonadherence:** 

**How Duchess avoids this:** We do not. Asynchronous exceptions are not recommended in modern code.

### Memory exhaustion from too many local references

[JNI reference states:](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#global_and_local_references)

> However, there are times when the programmer should explicitly free a local reference. Consider, for example, the following situations:
>
> * A native method accesses a large Java object, thereby creating a local reference to the Java object. The native method then performs additional computation before returning to the caller. The local reference to the large Java object will prevent the object from being garbage collected, even if the object is no longer used in the remainder of the computation.
> * A native method creates a large number of local references, although not all of them are used at the same time. Since the VM needs a certain amount of space to keep track of a local reference, creating too many local references may cause the system to run out of memory. For example, a native method loops through a large array of objects, retrieves the elements as local references, and operates on one element at each iteration. After each iteration, the programmer no longer needs the local reference to the array element.
>
> The JNI allows the programmer to manually delete local references at any point within a native method.

### Native references crossing threads

[JNI document states:](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#global_and_local_references)

> Local references are only valid in the thread in which they are created. The native code must not pass local references from one thread to another.

### Clear exceptions before invoking other JNI calls

> After an exception has been raised, the native code must first clear the exception before making other JNI calls. 

**Outcome of nonadherence:** 

**How Duchess avoids this:** Uh, do we? Certainly we internally propagate exceptions. What happens if you don't?

### Illegal argument types

[JNI document states](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#reporting_programming_errors):

> Reporting Programming Errors
>
> The JNI does not check for programming errors such as passing in NULL pointers or illegal argument types.
>
> The programmer must not pass illegal pointers or arguments of the wrong type to JNI functions. Doing so could result in arbitrary consequences, including a corrupted system state or VM crash.

**How Duchess avoids this:** Uh, do we? Certainly we internally propagate exceptions. What happens if you don't?

### Local references cannot be saved in global variables.

**Outcome of nonadherence:** Random crashes

**How Duchess avoids this:**

### Always check for exceptions (or return codes) on return from a JNI function. Always handle a deferred exception immediately you detect it.

**Outcome of nonadherence:** Unexplained exceptions or undefined behavior, crashes

**How Duchess avoids this:** End-users do not directly invoke JNI functions. Within Duchess, virtually all calls to JNI functions use the `EnvPtr::invoke` helper function which checks for exceptions. A small number use `invoke_unchecked` and require further audit.

### Ensure that array and string elements are always freed.

**Outcome of nonadherence:** Memory leak

**How Duchess avoids this:** 

### Ensure that you use the isCopy and mode flags correctly. See Copying and pinning.

Outcome of nonadherence: Memory leaks and/or heap fragmentation

### Local variable capacity

EnsureLocalCapacity

jint EnsureLocalCapacity(JNIEnv *env, jint capacity);

Ensures that at least a given number of local references can be created in the current thread. Returns 0 on success; otherwise returns a negative number and throws an OutOfMemoryError.

Before it enters a native method, the VM automatically ensures that at least 16 local references can be created.

For backward compatibility, the VM allocates local references beyond the ensured capacity. (As a debugging support, the VM may give the user warnings that too many local references are being created. In the JDK, the programmer can supply the -verbose:jni command line option to turn on these messages.) The VM calls FatalError if no more local references can be created beyond the ensured capacity.
LINKAGE:
Index 26 in the JNIEnv interface function table.
SINCE:

JDK/JRE 1.2

### Push Local Frame

-- what about interaction between jni libraries?

PushLocalFrame

jint PushLocalFrame(JNIEnv *env, jint capacity);

Creates a new local reference frame, in which at least a given number of local references can be created. Returns 0 on success, a negative number and a pending OutOfMemoryError on failure.

Note that local references already created in previous local frames are still valid in the current local frame.
LINKAGE:
Index 19 in the JNIEnv interface function table.
SINCE:

JDK/JRE 1.2