//! Exports common parts of the JDK.

#[cfg(not(doc))]
use crate as duchess;

duchess_macro::java_package! {
    package java.lang;

    class Object {
        Object();
        hashCode();
        equals(java.lang.Object);
        toString();
        notify();
        notifyAll();
    }

    class Record { }

    class String { }

    class CharSequence { }

    class Comparable { }

    class Cloneable { }

    class Iterable { }

    package java.lang.constant;

    class ConstantDesc { }

    class Constable { }

    package java.io;

    class Serializable { }

    package java.util;

    class List {
        add(E);
    }

    class ArrayList {
        ArrayList();
    }

    class Map {
        put(K, V);
    }

    class HashMap {
        HashMap();
    }

    class AbstractMap { }

    class RandomAccess { }

    class AbstractList { }

    class AbstractCollection { }

    class Collection { }

}
