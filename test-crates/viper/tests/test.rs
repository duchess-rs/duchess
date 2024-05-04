use duchess::prelude::*;
use java::lang::{Object, String};
use viper::scala::collection::immutable::Seq;
use viper::scala::collection::mutable::ArrayBuffer;
use viper::scala::Tuple2;
use viper::viper::carbon::CarbonVerifier;
use viper::viper::silicon::Silicon;
use viper::viper::silver::ast;
use viper::viper::silver::reporter::NoopReporter__;

/// Builds an empty Scala Seq of the specified type
fn empty_scala_seq<T: duchess::JavaObject>() -> impl IntoJava<Seq<T>> {
    ArrayBuffer::new().to_seq().assert_not_null()
}

#[test]
fn test_program_construction() -> duchess::Result<()> {
    duchess::Jvm::with(|jvm| {
        let domains = empty_scala_seq::<ast::Domain>();
        let fields = empty_scala_seq::<ast::Field>();
        let functions = empty_scala_seq::<ast::Function>();
        let predicates = empty_scala_seq::<ast::Predicate>();
        let methods = empty_scala_seq::<ast::Method>();
        let extension_member = empty_scala_seq::<ast::ExtensionMember>();
        let position = ast::NoPosition__::get_module();
        let info = ast::NoInfo__::get_module();
        let error_trafo = ast::NoTrafos__::get_module();
        let _program = ast::Program::new(
            domains,
            fields,
            functions,
            predicates,
            methods,
            extension_member,
            position,
            info,
            error_trafo,
        )
        .do_jni(jvm)?;
        Ok(())
    })
}

#[test]
fn test_carbon_construction() -> duchess::Result<()> {
    duchess::Jvm::with(|jvm| {
        let reporter = NoopReporter__::get_module();
        let debug_info = empty_scala_seq::<Tuple2<String, Object>>();
        let _carbon = CarbonVerifier::new(reporter, debug_info).do_jni(jvm)?;
        Ok(())
    })
}

#[test]
fn test_silicon_construction() -> duchess::Result<()> {
    duchess::Jvm::with(|jvm| {
        let _silicon = Silicon::new().do_jni(jvm)?;
        Ok(())
    })
}
