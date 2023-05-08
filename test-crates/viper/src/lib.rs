duchess::java_package! {
    package scala;
    class AnyVal { * }
    class Function1 {}
    class Tuple2<T1, T2> {}

    package scala.collection.immutable;
    interface Seq<A> {}

    package viper.silver.ast;
    class Bool {}
    class Int {}

    package viper.silver.frontend;
    class SilFrontend {}

    package viper.silver.reporter;
    interface Reporter {}
    class "NoopReporter$" {
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
