# Upcasts

The `Upcast` trait encodes extends / implements relations between classes and interfaces.
It is implemented both for direct supertypes as well as indirect (transitive) ones.
For example, if you have this Java class:

```java
class Foo extends Bar { }
class Bar implements Serializable { }
```

then the `Foo` type in Rust would have several `Upcast` impls:

* `Foo: Upcast<Bar>` -- because `Foo` extends `Bar`
* `Foo: Upcast<java::io::Serializable>` -- because `Bar` implements `Serializable`
* `Foo: Upcast<java::lang::Object>` -- because `Bar` extends `Object`

There is however one caveat.
We can only inspect the tokens presented to us.
And, while we could reflect on the Java classes directly, 
we don't know what subset of the supertypes the user has chosen to reflect into Rust. 
Therefore, we stop our transitive upcasts at the "water's edge" -- 
i.e., at the point where we encounter classes that are outside our package.

## Computing transitive upcasts

Transitive upcasts are computed in `upcasts.rs`. 
The process is very simple.
A map is seeded with each type `C` that we know about along its direct upcasts.
So, for the example above, this map would initially contain:

* `Foo => {Bar, Object}`
* `Bar => {Serializable, Object}`

we then iterate over the map and grow the entry for each class `C` with the supertypes of each class `D` that is extended by `C`.
So, for the example above, we would iterate to `Foo`, fetch the superclasses of `Bar`, and then union the into the set for `Foo`.

## Substitution

One caveat on the above is that we have to account for substitution.
If `Foo` extends `Baz<X>`, then we substitute `X` for the generic parameter of `Baz`.