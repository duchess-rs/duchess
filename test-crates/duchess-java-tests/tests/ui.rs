use ui_test::*;

fn main() -> color_eyre::eyre::Result<()> {
    std::env::set_var("CLASSPATH", "../target/java");

    // Tests can be blessed with `cargo test -- -- --bless`.
    let bless = std::env::args().any(|arg| arg == "--bless");

    let mut config = Config::default();
    config.root_dir = "tests/ui".into();

    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless;
    }

    // Place the build artifacts in the `target/ui` directory instead of in the
    // crate root.
    config.out_dir = Some("../target/ui".into());

    // Make sure we can depend on duchess itself in our tests
    config.dependencies_crate_manifest_path = Some("Cargo.toml".into());

    let test_name = std::env::var_os("TESTNAME");

    run_tests_generic(
        config,
        move |path| {
            test_name
                .as_ref()
                .and_then(|name| {
                    Some(path.components().any(|c| {
                        c.as_os_str()
                            .to_string_lossy()
                            .contains(&*name.to_string_lossy())
                    }))
                })
                .unwrap_or(true)
                && path.extension().map(|ext| ext == "rs").unwrap_or(false)
        },
        |_, _| None,
        status_emitter::TextAndGha,
    )
}
