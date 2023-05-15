use ui_test::*;

fn main() -> color_eyre::eyre::Result<()> {
    let bless = std::env::args().any(|arg| arg == "--bless");
    let mut config = Config::default();
    config.root_dir = "tests/ui".into();
    config.program.args.push("--crate-type".into());
    config.program.args.push("lib".into());

    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless;
    }

    config.dependencies_crate_manifest_path = Some("Cargo.toml".into());

    run_tests(config)
}
