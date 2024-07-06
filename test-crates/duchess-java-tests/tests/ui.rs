use diagnostics::Diagnostics;
use std::path::Path;
use ui_test::*;

fn run_rust_tests(
    test_group_name: &str,
    test_path: &Path,
    per_file_file_config: impl Fn(&mut Config, &Path, &[u8]) + Sync,
) -> color_eyre::eyre::Result<()> {
    // Tests can be blessed with `DUCHESS_BLESS=1``
    let bless = std::env::var("DUCHESS_BLESS").is_ok();

    let mut config = Config {
        ..Config::rustc(test_path)
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

    config.out_dir = Path::new("..").join("target");

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
        per_file_file_config,
        (
            status_emitter::Text::from(args.format),
            ui_test::status_emitter::Gha::<true> {
                name: test_group_name.to_string(),
            },
        ),
    )
}

fn run_java_tests(test_group_name: &str) -> color_eyre::eyre::Result<()> {
    let test_name = std::env::var_os("TESTNAME");

    let mut java = CommandBuilder::cmd("../target/debug/java_wrapper");
    java.envs = vec![
        (
            "LD_LIBRARY_PATH".into(),
            Some("../target/tests/java-to-rust/rust-libraries".into()),
        ),
        (
            "CLASSPATH".into(),
            Some("../target/tests/java-to-rust/java".into()),
        ),
    ];
    java.args = vec!["-Djava.library.path=../target/tests/java-to-rust/rust-libraries".into()];

    let java_config = Config {
        host: Some("host".to_string()),
        target: None,
        root_dir: Path::new("tests").join("java-to-rust/java"),
        program: java,
        output_conflict_handling: OutputConflictHandling::Ignore,
        bless_command: None,
        out_dir: std::env::var_os("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"))
            .join("tests/java-to-rust/java"),
        skip_files: Vec::new(),
        filter_files: Vec::new(),
        threads: None,
        list: false,
        run_only_ignored: false,
        filter_exact: false,
        comment_defaults: ui_test::per_test_config::Comments::default(),
        custom_comments: Default::default(),
        diagnostic_extractor: |_path: &Path, diagnostic_msg: &[u8]| Diagnostics {
            rendered: diagnostic_msg.to_vec(),
            messages: vec![],
            messages_from_unknown_file_or_line: vec![],
        },
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
        |_c: &mut Config, _p: &Path, _f: &[u8]| {},
        (
            status_emitter::Text::from(Args::test()?.format),
            ui_test::status_emitter::Gha::<true> {
                name: test_group_name.to_string(),
            },
        ),
    )
}

fn main() -> color_eyre::eyre::Result<()> {
    std::env::set_var("CLASSPATH", "../target/java");
    run_rust_tests(
        "rust ui tests",
        &Path::new("tests").join("rust-to-java"),
        default_per_file_config,
    )?;

    // rust classes in rust-to-java/rust-libraries are rust files that
    // are compiled into shared object libraries which are loaded by java tests
    let library_per_file_config =
        |config: &mut Config, _path: &Path, _file_contents: &[u8]| -> () {
            config.program.args.push("--crate-type=cdylib".into());
            config.program.envs.push((
                "CLASSPATH".into(),
                Some("../target/tests/java-to-rust/java".into()),
            ));
        };

    std::env::set_var("CLASSPATH", "../target/rust-to-java/java");
    run_rust_tests(
        "compile rust libraries for java tests",
        &Path::new("tests").join("java-to-rust/rust-libraries"),
        library_per_file_config,
    )?;

    // run java tests that depend on the above libraries
    run_java_tests("java ui tests")
}
