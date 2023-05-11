duchess::java_package! {
    package java.lang;
    class Object {}

    package scala;
    class AnyVal { * }
    class Function1 {}
    class Tuple2<T1, T2> {}

    package scala.collection;
    interface IterableOnceOps<A, CC, C> {
        default scala.collection.immutable.Seq<A> toSeq();
    }
    interface IterableOps<A, CC, C> extends scala.collection.IterableOnceOps<A, CC, C> {}
    interface SeqOps<A, CC, C> extends scala.collection.IterableOps<A, CC, C> {}
    interface StrictOptimizedSeqOps<A, CC, C> extends scala.collection.SeqOps<A, CC, C> {}

    package scala.collection.immutable;
    interface Seq<A> {}

    package scala.collection.mutable;
    class ArrayBuffer<A>
        implements scala.collection.StrictOptimizedSeqOps<A, scala.collection.mutable.ArrayBuffer, scala.collection.mutable.ArrayBuffer<A>>
    {
        scala.collection.mutable.ArrayBuffer();
    }

    package viper.silver.ast;
    class Bool {}
    class Int {}

    package viper.silver.frontend;
    class SilFrontend {}

    package viper.silver.reporter;
    interface Reporter {}
    class "NoopReporter$" implements viper.silver.reporter.Reporter {
        static viper.silver.reporter."NoopReporter$" "MODULE$";
    }

    package viper.silicon;
    class Silicon {
        viper.silicon.Silicon();
    }

    package viper.carbon;
    class CarbonVerifier {
        viper.carbon.CarbonVerifier(viper.silver.reporter.Reporter, scala.collection.immutable.Seq<scala.Tuple2<java.lang.String, java.lang.Object>>);
    }
}
