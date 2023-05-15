use ui_test::*;

fn main() -> color_eyre::eyre::Result<()> {
    // Tests can be blessed with `cargo test -- -- --bless`.
    let bless = std::env::args().any(|arg| arg == "--bless");

    let mut config = Config::default();
    config.root_dir = "tests/ui".into();

    // Don't require `fn main()`. If this becomes desirable, we need to either split
    // the test suite into two folders, or require `fn main()` in all tests.
    config.program.args.push("--crate-type".into());
    config.program.args.push("lib".into());

    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless;
    }

    // Make sure we can depend on duchess itself in our tests
    config.dependencies_crate_manifest_path = Some("Cargo.toml".into());

    run_tests(config)
}
