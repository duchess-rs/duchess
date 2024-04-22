use std::path::Path;
use ui_test::*;

fn main() -> color_eyre::eyre::Result<()> {
    std::env::set_var("CLASSPATH", "../target/java");

    // Tests can be blessed with `DUCHESS_BLESS=1``
    let bless = std::env::var("DUCHESS_BLESS").is_ok();

    let mut config = Config {
        ..Config::rustc(Path::new("tests").join("ui"))
    };

    if std::env::var("RUSTFLAGS")
        .unwrap_or_default()
        .contains("instrument-coverage")
    {
        config.program.args.push("-C".into());
        config.program.args.push("instrument-coverage".into());
    }

    let args = Args::test()?;

    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless;
    }

    // Place the build artifacts in the `../target/ui` directory instead of in the
    // crate root.
    config.out_dir = Path::new("..").join("target").join("ui");

    // Make sure we can depend on duchess itself in our tests
    config.dependencies_crate_manifest_path = Some(Path::new("Cargo.toml").into());

    let test_name = std::env::var_os("TESTNAME");

    let text = if args.quiet {
        ui_test::status_emitter::Text::quiet()
    } else {
        ui_test::status_emitter::Text::verbose()
    };

    run_tests_generic(
        vec![config],
        args,
        move |path, _, _| {
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
        default_per_file_config,
        (
            text,
            ui_test::status_emitter::Gha::<true> {
                name: "ui tests".into(),
            },
        ),
    )
}
