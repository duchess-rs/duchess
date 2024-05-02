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
    } else {
        config.output_conflict_handling = OutputConflictHandling::Ignore;
    }

    // Place the build artifacts in the `../target/ui` directory instead of in the
    // crate root.
    config.out_dir = Path::new("..").join("target").join("ui");

    // Make sure we can depend on duchess itself in our tests
    config
        .comment_defaults
        .base()
        .set_custom("dependencies", dependencies::DependencyBuilder::default());

    let test_name = std::env::var_os("TESTNAME");

    let text = status_emitter::Text::from(args.format);

    config.with_args(&args);

    run_tests_generic(
        vec![config],
        move |path, _| {
            path.extension().filter(|ext| *ext == "rs")?;
            Some(
                test_name
                    .as_ref()
                    .and_then(|name| {
                        Some(path.components().any(|c| {
                            c.as_os_str()
                                .to_string_lossy()
                                .contains(&*name.to_string_lossy())
                        }))
                    })
                    .unwrap_or(true),
            )
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
