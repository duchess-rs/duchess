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
    class ArrayBuffer<A> implements scala.collection.StrictOptimizedSeqOps<
        A, scala.collection.mutable.ArrayBuffer, scala.collection.mutable.ArrayBuffer<A>
    > {
        public scala.collection.mutable.ArrayBuffer();
    }

    package viper.silver.ast;
    class Bool {}
    class Domain {}
    class "NoTrafos$" implements viper.silver.ast.ErrorTrafo {
        public static viper.silver.ast."NoTrafos$" "MODULE$";
    }
    class ExtensionMember {}
    class Field {}
    class Function {}
    class Int {}
    class Method {}
    class "NoPosition$" implements viper.silver.ast.Position {
        public static viper.silver.ast."NoPosition$" "MODULE$";
    }
    class "NoInfo$" implements viper.silver.ast.Info {
        public static viper.silver.ast."NoInfo$" "MODULE$";
    }
    class Predicate {}
    class Program {
        public viper.silver.ast.Program(
            scala.collection.immutable.Seq<viper.silver.ast.Domain>,
            scala.collection.immutable.Seq<viper.silver.ast.Field>,
            scala.collection.immutable.Seq<viper.silver.ast.Function>,
            scala.collection.immutable.Seq<viper.silver.ast.Predicate>,
            scala.collection.immutable.Seq<viper.silver.ast.Method>,
            scala.collection.immutable.Seq<viper.silver.ast.ExtensionMember>,
            viper.silver.ast.Position,
            viper.silver.ast.Info,
            viper.silver.ast.ErrorTrafo
        );
    }
    interface ErrorTrafo {}
    interface Info {}
    interface Position {}

    package viper.silver.frontend;
    class SilFrontend {}

    package viper.silver.reporter;
    interface Reporter {}
    class "NoopReporter$" implements viper.silver.reporter.Reporter {
        public static viper.silver.reporter."NoopReporter$" "MODULE$";
    }

    package viper.silicon;
    class Silicon {
        public viper.silicon.Silicon();
    }

    package viper.carbon;
    class CarbonVerifier {
        public viper.carbon.CarbonVerifier(
            viper.silver.reporter.Reporter,
            scala.collection.immutable.Seq<scala.Tuple2<java.lang.String, java.lang.Object>>
        );
    }
}
