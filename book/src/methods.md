# Methods

When you use duchess, you invoke methods via nice syntax like

```rust
java_list.get(0).to_string().execute()
//               ^^^^^^^^^
// This is the method we are discussing here
```

How does this actually work (and why)?

## Complication #1: Methods are invokable on more than just the object

*Part* of our setup is that define relatively ordinary looking inherent methods
on the type that defines the method, e.g.:

```rust
impl Object {
    fn to_string(&self) -> impl JavaMethod<String> { /* tbd */ }
}
```

This method will be invoked when people have a variable `o: &Object`
or `o: Global<Object>` and they write `o.to_string()`.
But it won't support our example of `java_list.get(0).to_string()`,
because `java_list.get(0)` returns a [`JvmOp`], not an `Object`. 
So, to define a method on `Object`,
we need a way to put methods onto **any [`JvmOp`] that outputs an `Object`**.

[`JvmOp`]: ./jvm_operations.md

## Complication #2: Overridden or implemented methods create ambiguity

There are some complications in getting `.` syntax to work.
We want users to be able to write `m.foo()` but, in Java,
the same method `foo` is often defined in multiple places,
particularly when it is overridden:

* on the class type itself
* maybe on supertypes, if it is overridden
* maybe on interfaces, if it is an interface method

We don't want users to get ambiguity errors when calling `foo`.
We want them to get the most specific version of the method.
This is important not because we'll call the wrong thing -- the JVM handles the virtual dispatch.
But it can impact the return type.

## Complication #3: We don't know the reflected signatures of all methods on every type

When generating code for one class `X`, it may have a supertype `Y` that is outside our `java_package` macro invocation.
Or, it may have methods that return a value of type `Z` that is outside our `java_package` macro invocation.
While we can leverage Java reflection to know the *Java* methods of `Y` and `Z`, that doesn't tell us what the *Rust* methods are.
This is because users can subset the methods of `Y` and `Z` as well as making other changes, 
such as renaming them to avoid overload conflicts.
So we have to support method dispatch with an incomplete view of the methods of `Y` and `Z`.

As one example of how this can be tricky, suppose that we attempted to resolve complication #2 by
only generating methods on the "root location" that defined them.

## Complication #4: Extension traits are not ergonomic

[complication 4]: #compilation-4-extension-traits-are-not-ergonomic

Ideally, we would define all methods as some kind of inherent method so that users do not need to import extension traits or deal with special-case preludes.

## Outline and TL;DR

This section gives a brief overview of all the pieces of our solution and how they fit together
In the following sections, we are going to walk through each part of the solution step by step.

* Inherent associated functions on the object types
    * To support fully qualified dispatch, we add inherent associated functions (*not* methods, so no `self` parameter) to each Java class/interface. These are used if you write something like `Object::to_string(o)`. The parameter `o` must be an `impl IntoJava<Object>`.
* Concept: newtyped references and `FromRef`
    * The design below leans heavily on a pattern of newtyped references.
    * The idea is that given some reference `&X`, we define types like `struct Wrapper<X> { x: X }` with `#[repr(transparent)]`. The transparent representation ensures that `X` and `Wrapper<X>` have the same layout in memory and are treated equivalently in ABIs and the like.
    * Now we can safely transmute from `&X` to `&Wrapper<X>`.
* Concept: method resolution order
    * *Method resolution order* is defined using Python's [C3] algorithm. It is an ordering of the transitive supertypes (classes, interfaces) of `C` such that, if `X` extends `Y`, then `X` appears before `Y` in the MRO.
* Modeling method resolution order (MRO) for a class/interface `C` with `ViewAs` structs
    * For each class/interface `C`, define an "view-as struct" that looks like `ViewAsC<J, N>`
        * A reference of type `&ViewAsC<J, N>` indicates a reference of type `&J` that is being "viewed as" a reference of type `&C`
        * The `ViewAs` structs is a "newtyped reference" from `J`, and so `ViewAsC<J, N>: FromRef<J>`.
        * The `N` parameter indicates the "view" struct for the next type in the *method resolution order* when upcasting from `J`

            * FIXME: We could probably refactor `N` away so that we just have `AsC<J>` and we use an auxiliary trait like `J: MRO<C, Next = N>`.
        * `ViewAsC<J, N>` derefs to `N`.
    * The `ViewAsC` structs are not nameable directly; instead the `JavaObject` trait includes an associated type `<C as JavaObject>::ViewOn<J>` that maps to `ViewAsC<J, M>` where `M` is the default MRO.
    * Define deref from `C` to `C::ViewOn<C>` (i.e., `ViewAsC<C, M>`).
    * Example:
        * Given `Foo extends Bar, Baz`, the type `Foo` would deref to
            * `ViewAsFoo<Foo, ViewAsBar<Foo, ViewAsBaz<Foo, ()>>>`, which in turn derefs to
            * `ViewAsBar<Foo, >`, which in turn derefs to
            * `ViewAsBaz<Foo, ()>`, which in turn derefs to
            * `()`
* Inherent methods on `ViewAs` structs
    * Next we add inherent methods like `fn to_string(&self) -> impl JavaMethod<java::lang::String> + '_`  to the view as structs in which those methods are defined.
        * In the case of `to_string`, this would appear on `ViewAsObject<J, N>`, but also other classes that override `toString` 
        * The definition of this function just calls the inherent associated function `Object::to_string`
    * Rust's method dispatch will walk through the MRO, selecting the best method to use and invoking it
* Invocations on other [`JvmOp`] values with `OfOpAs` structs
    * To support chained dispatch, we also need to support invocations on other [`JvmOp`] values.
    * We create a "view op as" struct that works exactly like `ViewAs`, e.g., `OfOpAsC<O, N>`
        * The difference is that `O` here is not a `JavaObject` type but rather a `impl IntoJava<J>` for some `J`
    * The `N` parameter models the MRO in an analogous way to `ViewAs` structs
    * Inherent methods are defined on the `OfOpAs` structs
    * Example:
        * Given `Foo extends Bar, Baz`, and some op `O` that produces a `Foo`, `O` would deref to
            * `OfOpAsFoo<O, OfOpAsBar<O, OfOpAsBaz<O, ()>>>`
            * `OfOpAsBar<O, OfOpAsBaz<O, ()>>`
            * `OfOpAsBaz<O, ()>`
            * `()`
        * ...and thus users can invoke `produce_foo().some_foo_method()`

<a name="assoc-fn"></a>

## Inherent associated functions on the object types

The first step is to create a "fully qualified" notation for each Java method:

```rust
impl Object {
    fn to_string(
        this: impl IntoJava<Object>
    ) -> impl JavaMethod<java::lang::String> {
        ...
    }
}
```

This function does not take a `self` parameter and so it can only be invoked using fully qualified form, e.g., `Object::to_string(foo)`.

## Concept: newtyped references and `FromRef`

The next step is that we define a trait `FromRef` that we will use to define a pattern called 'newtyped references'.
The idea is that we want to be able to take a reference `&J` and convert it into a *view* on that reference `&View<J>`,
where `View<J>` has the same data as `J` but defines inherent methods.

We'll create a trait `FromRef` to use for this pattern,
where `View<J>: FromRef<J>` indicates that a view `&View<J>` can be constructed from a `&J` reference:

```rust
pub trait FromRef<J> {
    fn from_ref(t: &J) -> &Self;
}
```

A view struct is just a newtype on the underlying `J` type but with `#[repr(transparent)]`:

```rust
#[repr(transparent)]
pub struct View<J> {
    this: J,
}
```

The `#[repr(transparent)]` attribute ensures that `J` and `View<J>` have the same layout in memory
and are treated equivalently in ABIs and the like. 
Thanks to this, we can implement `FromRef` like so:

```rust
impl FromRef<J> for View<J> {
    fn from_ref(t: &J) -> &Self {
        // Safe because of the `#[repr(transparent)]` attribute
        unsafe { std::mem::transmute(t) }
    }
}
```

## Concept: Method resolution order (MRO)

The *method resolution order* for a type `T` is an ordered list of its transitive supertypes such that, given two types `X` and `Y` in the list, if `X` extends `Y` then `X` appears before `Y`. This ensures that if we search linearly down the list, we will find the "most refined" version of a method first. We define the MRO for a type `T` using Python's [C3] algorithm.

[c3]: https://www.python.org/download/releases/2.3/mro/

## Modeling method resolution order (MRO) for a class/interface `C` with `ViewAs` structs

For each class `X`, we define a *`ViewAsObj` struct* `ViewAsXObj<J, N>`:

```rust
#[repr(transparent)]
struct ViewAsXObj<J, N> {
    this: J,
    phantom: PhantomData<N>,
}
```

The class has two type parameters:

* The parameter `J` identifies the original type from which we created the view; this will always be some sutype of `X`.
* The `N` parameter represents the remainder of `J`'s method resolution order.

### Deref chain

Each ViewAsObj struct includes a Deref that derefs to N:

```rust
impl<J, N> Deref for ViewAsXObj<J, N> {
    type Target = N;

    fn deref(&self) -> 
} 
```

### Chaining ViewAsObj structs

So given `interface Foo extends Bar, Baz`, the type `Foo` would deref to

```rust
ViewAsFooObj<Foo, ViewAsBarObj<Foo, ViewAsBazObj<Foo, ()>>>
//           ---  -----------  ----------------------------------
//           X    J            N
```

Each `ViewAs` struct derefs to its `N` parameter, 
so `ViewAsFooObj<Foo, ViewAsBarObj<Foo, ...>>` would deref to `ViewAsBarObj<Foo, ...>` 
and so forth.

## The `FromRef` trait

Each op struct implements a trait `FromRef<J>`:

```rust
trait FromRef<J> {
    fn from_ref(r: &J) -> &Self;
}
```

The `from_ref` method allows constructing an op struct from an `&J` reference.
Implementing this method requires a small bit of unsafe code, 
leveraging the `repr(transparent)` attribute on each op struct:

```rust
impl<J> FromRef<J> for ObjectOp<J>
where
    J: IntoJava<Foo>,
{
    fn from_ref(r: &J) -> &Self {
        // Safe because ObjectOp<J> shares representation with J:
        unsafe { std::mem::transmute(r) }
    }
}
```

## Methods on ViewAsObj structs

The `ViewAsObj` struct for a given Java type 
also has inherent methods for each Java method.
These are implemented by invoking the [fully qualified inherent functions]().
For example, the ViewAsObj struct for `Object` includes a `to_string` method like so:

```rust
impl<J, N> ViewAsObjectObj<J, N>
where
    J: Upcast<Object>,
{
    pub fn to_string(&self) -> impl JavaMethod<java::lang::String> + '_ {
        java::lang::Object::to_string(&self.this)
    }
}
```

## Naming op structs: the `JavaObject::OfOp` associated type

We don't want `ViewAsObj` structs to be publicly visible.
So we create them inside of a `const _: () = { .. }` block.
But we do need *some* way to name them.
We expose them via associated types of the `JavaView` trait:

```rust
trait JavaView {
    type OfObj<J>: FromRef<J>;
    type OfObjWith<J, N>: FromRef<J>
    where
        N: FromRef<J>;
}
```

The `OfObj` associated type in particular provides the "default value" for `N` that defines the MRO.
The `OfObjWith` is used to supply an explicit `N` value. For example:

```rust
const _: () = {
    struct ViewAsFooObj<J, C> { ... }
    
    impl JavaView for Foo {
        type OfObj<J> = ViewAsFooObj<Foo, Bar::OfObjWith<Foo, Baz::OfObjWith<Foo, ()>>>;
        //              ------------ ---  --------------------------------------------
        //                |          |    Method resolution order      
        //                |          Original type we are viewing onto (i.e., Self)
        //              The ViewAsFoo object
    }
}
```

## `ViewAsOp` structs

The `ViewAsObj` structs allow you to invoke methods on a java object reference like `s: &java::lang::String`.
But they do not allow you to invoke methods on some random [`JvmOp`] that happens to *return* a string.
For that, we create a very similar set of `ViewAsOp` structs:

```rust
#[repr(transparent)]
struct ViewAsXOp<J, N> {
    this: J,
    phantom: PhantomData<N>,
}
```

These `ViewAsOp` structs look exactly the same, but the `J` here is not a java object but rather a [`JvmOp`].
Like the `ViewAsObj` structs, they have inherent methods that call to the fully qualified inherent methods.
But the signature is slightly different; it is a `&self` method, but the `impl JavaMethod` that is returned does not capture the `self` reference.
Instead, it copies the `self.this` out. 
This relies on the fact that all [`JvmOp`] values are `Copy`.

```rust
impl<J, N> ViewAsObjectObj<J, N>
where
    J: IntoJava<Object>,
{
    pub fn to_string(&self) -> impl JavaMethod<java::lang::String> {
        java::lang::Object::to_string(self.this)
    }
}
```

`ViewAsOp` structs are exposed through associated types on `JavaView` just like `ViewAsObj` structs.

#### Q: Why not a `self` method?

You might wonder why we take a `&self` method and then copy out rather than just taking `self`. 
The reason is that the `ViewAsObjectObj` traits are the output from `Deref` impls


### Deref impls on ops

We also have to add a `Deref` impl to each of the op structs.

```rust
struct SomeOp { }

impl JvmOp for SomeOp {
    type Output<'jvm> = Local<'jvm, Foo>;
}

impl Deref for SomeOp {
    type Target = <Foo as JavaView>::OfOp<SomeOp>;

    fn deref(&self) -> &Self::Target {
        FromRef::from_ref(self)
    }
}
```

