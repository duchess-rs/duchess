use std::path::Path;
use diagnostics::Diagnostics;
use ui_test::*;

fn run_rust_tests(text: status_emitter::Text) -> color_eyre::eyre::Result<()> {
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

fn run_java_tests(text: status_emitter::Text) -> color_eyre::eyre::Result<()> {
    let test_name = std::env::var_os("TESTNAME");

    let mut java = CommandBuilder::cmd("src/java_wrapper");
    java.envs = vec![
        ("LD_LIBRARY_PATH".into(), Some("../target/ui/tests/ui/".into())),
        ("CLASSPATH".into(), Some("../target/tests/java_ui".into()))
    ];

    let java_config = Config {
        host: Some("host".to_string()),
        target: None,
        root_dir: Path::new("tests").join("java_ui"),
        program: java,
        output_conflict_handling: OutputConflictHandling::Ignore,
        bless_command: None,
        out_dir: std::env::var_os("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"))
            .join("java_ui"),
        skip_files: Vec::new(),
        filter_files: Vec::new(),
        threads: None,
        list: false,
        run_only_ignored: false,
        filter_exact: false,
        comment_defaults: ui_test::per_test_config::Comments::default(),
        custom_comments: Default::default(),
        diagnostic_extractor: |_path: &Path, diagnostic_msg: &[u8]| {
            Diagnostics {
                rendered: diagnostic_msg.to_vec(),
                messages: vec![],
                messages_from_unknown_file_or_line: vec![]
            }
        }
    };

    run_tests_generic(
        vec![java_config],
        move |path, _| {
            path.extension().filter(|ext| *ext == "java")?;
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
        |_c: &mut Config, _p: &Path, _f: &[u8]| {
        },
        (
            text,
            ui_test::status_emitter::Gha::<true> {
                name: "java ui tests".into(),
            },
        ),
    )
}

fn main() -> color_eyre::eyre::Result<()> {
    let args = Args::test()?;

    run_rust_tests(status_emitter::Text::from(args.format))?;

    // java tests must run after rust tests
    // the java tests assume the rust tests will compile
    // any shared libraries needed for the java tests
    run_java_tests(status_emitter::Text::from(args.format))
}
