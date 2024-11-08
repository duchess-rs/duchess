use std::path::{Path, PathBuf};

use anyhow::Context;
use duchess_reflect::reflect::JavapReflector;
use java_compiler::JavaCompiler;

mod code_writer;
mod files;
mod impl_java_trait;
mod java_compiler;
mod java_package_macro;
mod re;
mod shim_writer;

pub use duchess_reflect::config::Configuration;

/// Build Rs configuration for duchess.
/// To use duchess you must invoke [`DuchessBuildRs::execute`][].
///
/// # Example
///
/// The simplest build.rs is as follows.
///
/// ```rust,no_run
/// use duchess_build_rs::DuchessBuildRs;
///
/// fn main() -> anyhow::Result<()> {
///     DuchessBuildRs::new().execute()
/// }
/// ```
pub struct DuchessBuildRs {
    configuration: Configuration,
    src_path: PathBuf,
    in_cargo: bool,
    temporary_dir: Option<PathBuf>,
}

impl Default for DuchessBuildRs {
    fn default() -> Self {
        DuchessBuildRs {
            configuration: Configuration::default(),
            src_path: PathBuf::from("."),
            in_cargo: std::env::var("CARGO").is_ok() && std::env::var("OUT_DIR").is_ok(),
            temporary_dir: None,
        }
    }
}

impl DuchessBuildRs {
    /// Create a new DuchessBuildRs instance.
    /// Equivalent to `DuchessBuildRs::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Customize the JDK configuration (e.g., CLASSPATH, etc).
    pub fn with_configuration(mut self, configuration: Configuration) -> Self {
        self.configuration = configuration;
        self
    }

    /// Configure the path where Rust sources are found.
    /// The default is `.`.
    /// We will automatically search all subdirectories for `.rs` files.
    pub fn with_src_path(mut self, src_path: PathBuf) -> Self {
        self.src_path = src_path;
        self
    }

    /// Where to store temporary files (generated java, class files that are not being exported).
    /// If unset, a fresh temporary directory is created that will be wiped up later.
    pub fn with_temporary_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.temporary_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Execute the duchess `build.rs` processing.
    ///
    /// Detects uses of duchess build macros and derives
    /// and generates necessary support files in the `OUT_DIR` side.
    ///
    /// NB: Duchess macros must be written like `duchess::name!` or `#[duchess::name]`.
    pub fn execute(self) -> anyhow::Result<()> {
        // TODO: Russell November 8th.
        // Things are very close to done.
        // We need to capture things like #[java(java.lang.Throwable)] and reflect those to store the types
        // in the reflection cache.

        // You can see this failure currently if you run `just test`.

        let compiler = &JavaCompiler::new(&self.configuration, self.temporary_dir.as_ref())?;
        eprintln!(
            "looking for files in {:?} (total: {})",
            self.src_path,
            files::rs_files(&self.src_path).count()
        );
        let mut reflector = JavapReflector::new(&self.configuration);
        for rs_file in files::rs_files(&self.src_path) {
            let rs_file = rs_file?;
            let mut watch_file = false;

            eprintln!("looking for java macros in {:?}", rs_file.path);
            watch_file |= java_package_macro::process_file(&rs_file, &mut reflector)?;

            for capture in re::impl_java_interface().captures_iter(&rs_file.contents) {
                let std::ops::Range { start, end: _ } = capture.get(0).unwrap().range();
                impl_java_trait::process_impl(compiler, &rs_file, start)
                    .with_context(|| "failed to parse impl")?;
                watch_file = true;
            }

            if watch_file && self.in_cargo {
                println!("cargo:rerun-if-changed={}", rs_file.path.display());
            }
        }
        let out_dir = std::env::var("OUT_DIR").unwrap();
        eprintln!("dumping {} classes to {out_dir}", reflector.len());
        reflector.dump_to(Path::new(&out_dir))?;
        println!("cargo::rustc-env=DUCHESS_OUT_DIR={}", out_dir);
        if let Some(classpath) = self.configuration.classpath() {
            println!("cargo::rustc-env=CLASSPATH={}", classpath);
        } else {
            println!("cargo::rustc-env=CLASSPATH={}", out_dir);
        }
        Ok(())
    }
}
