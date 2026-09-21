//@run
use duchess::prelude::*;

// No `use duchess::java` here. This file's `java_package!` generates a local
// top-level `java` module, which would conflict with the explicit import. Plain
// `java::util::...` below refers to the local `mod java`, while
// `duchess::java::...` reaches the prelude.

// Declaring `package java.util` while referencing prelude classes from the same
// package exercises mixed local/prelude resolution: `ArrayList`/`HashMap`
// resolve relatively, while `List` (return position) and `Map` (`implements`
// and argument position) resolve to `duchess::java::util::...`.
duchess::java_package! {
    package java.util;

    public class java.util.ArrayList<E> {
        public java.util.ArrayList();
        public boolean add(E);
        public java.util.List<E> subList(int, int);
    }

    public class java.util.HashMap<K, V> implements java.util.Map<K, V> {
        public java.util.HashMap();
        public int size();
        public V put(K, V);
        public void putAll(java.util.Map<? extends K, ? extends V>);
    }
}

type JavaString = duchess::java::lang::String;

fn main() -> duchess::Result<()> {
    duchess::Jvm::builder().try_launch()?;

    let list: Java<java::util::ArrayList<JavaString>> = java::util::ArrayList::new().execute()?;
    assert!(list.add("hello").execute()?);
    assert!(list.add("world").execute()?);

    // `subList` returns the prelude `List`; its methods come from the prelude.
    let sub: Java<duchess::java::util::List<JavaString>> =
        list.sub_list(0, 1).assert_not_null().execute()?;
    assert_eq!(sub.size().execute()?, 1);
    assert!(!sub.is_empty().execute()?);

    // `putAll` takes the prelude `Map`; `&HashMap` upcasts to it via the
    // declared `implements`.
    let map1: Java<java::util::HashMap<JavaString, JavaString>> =
        java::util::HashMap::new().execute()?;
    let map2: Java<java::util::HashMap<JavaString, JavaString>> =
        java::util::HashMap::new().execute()?;
    let _: Option<Java<JavaString>> = map1.put("a", "1").execute()?;
    let _: Option<Java<JavaString>> = map2.put("b", "2").execute()?;
    map1.put_all(&map2).execute()?;
    assert_eq!(map1.size().execute()?, 2);
    assert_eq!(map2.size().execute()?, 1);

    Ok(())
}
