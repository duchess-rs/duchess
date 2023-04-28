fn main() {
    lalrpop::process_root().unwrap();

    // Recompile the crate when the lalrpop grammar changes
    println!("cargo:rerun-if-changed=src");

    // Procedural macros currently do not automatically rerun when the environment variables on
    // which they depend change. So, as a workaround we force a recompilation.
    // See issue: https://github.com/duchess-rs/duchess/issues/7
    println!("cargo:rerun-if-env-changed=CLASSPATH");
    println!("cargo:rerun-if-env-changed=DUCHESS_DEBUG");
}
