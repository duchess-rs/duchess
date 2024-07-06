use std::{
    path::{Path, PathBuf},
    process::Command,
};

use walkdir::WalkDir;

// This controls where this script writes built files
const TARGET_PATH: &str = "../target";

fn main() -> std::io::Result<()> {
    // Rerun java build if any source file changes, but then we'll check each file individually below
    let java_source_paths = vec!["java", "tests/java-to-rust/java"];
    for source_path in java_source_paths.iter() {
        println!("cargo:rerun-if-changed={}", source_path);
    }
    println!("cargo:rustc-env=CLASSPATH=target/java:target/tests/java-to-rust/java");

    let target_dir = Path::new(TARGET_PATH);

    for source_path in java_source_paths {
        for entry_result in WalkDir::new(source_path) {
            let entry = entry_result?;

            if let Some(extension) = entry.path().extension() {
                if extension == "java" {
                    // check if the class file doesn't exist or is older
                    let source = entry.into_path();

                    // The target class file is basically the same path as the Java source file, relative to the target
                    // directory
                    let target = target_dir.join(source.clone()).with_extension("class");

                    let build_file = BuildFile { source, target };

                    if !file_up_to_date(&build_file)? {
                        build_java(&build_file, source_path)?;
                    }
                }
            }
        }
    }

    Ok(())
}

// A simple holder for state on a given file
#[derive(Debug)]
struct BuildFile {
    source: PathBuf,
    target: PathBuf,
}

/// Determines whether the target file exists and is up-to-date by checking the last modified timestamp
fn file_up_to_date(BuildFile { source, target }: &BuildFile) -> std::io::Result<bool> {
    Ok(target.exists() && source.metadata()?.modified()? <= target.metadata()?.modified()?)
}

/// Executes javac to build the specified file
fn build_java(input: &BuildFile, source_path: &str) -> std::io::Result<()> {
    // Class files will hav the same path structure as the sources, relative to the target dir
    let javac_target_dir = Path::new(TARGET_PATH).join(source_path);

    let output = Command::new("javac")
        .args([
            "-d", // Specify the target directory for class files. Javac will create all parents if needed
            &javac_target_dir.display().to_string(),
            "-sourcepath", // Specify where to find other source files (e.g. dependencies)
            source_path,
            input.source.to_str().unwrap(), // assuming that we're not dealing with weird filenames
        ])
        .output()?;

    if !output.status.success() {
        let mut stdout: String =
            String::from_utf8(output.stdout).expect("Unable to parse javac output");
        stdout.push_str(String::from_utf8(output.stderr).unwrap().as_str());

        panic!("Failed to build {:?}: {}", input.source, stdout);
    }

    Ok(())
}
