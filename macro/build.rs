fn main() {
    lalrpop::process_root().unwrap();

    // Procedural macros currently do not automatically rerun when the environment variables on
    // which they depend change. So, as a workaround we force a recompilation.
    // See issue: https://github.com/duchess-rs/duchess/issues/7
    println!("cargo:rerun-if-env-changed=CLASSPATH");
}
