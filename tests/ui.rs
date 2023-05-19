use ui_test::*;

fn main() -> color_eyre::eyre::Result<()> {
    // Tests can be blessed with `cargo test -- -- --bless`.
    let bless = std::env::args().any(|arg| arg == "--bless");

    let mut config = Config::default();
    config.root_dir = "tests/ui".into();

    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless;
    }

    // Place the build artifacts in the `target/ui` directory instead of in the
    // crate root.
    config.out_dir = Some("target/ui".into());

    // Make sure we can depend on duchess itself in our tests
    config.dependencies_crate_manifest_path = Some("Cargo.toml".into());

    run_tests(config)
}
