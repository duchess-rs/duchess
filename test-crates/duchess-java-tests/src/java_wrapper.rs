// When ui_test runs the given binary, it passes the the binary the path to the test case
// java expects to be given the path to the class file in Java package format
//
// For example, ui_test will call the java application as:
// `java test/java_ui/java_test/JavaTestClass.java`
//
// but java expects to be invoked as
// `java test.java_ui.java_test.JavaTestClass`
//
// This wrapper script converts the given path into the expected java format
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let path_to_java_file = args.pop().unwrap();

    let path_converted_to_package_fmt = Path::new(&path_to_java_file)
        .strip_prefix("tests/java_ui")
        .unwrap()
        .with_extension("")
        .iter()
        .map(OsStr::to_str)
        .map(Option::unwrap)
        .collect::<Vec<&str>>()
        .join(".");

    args.push(path_converted_to_package_fmt);

    let output = Command::new("java").args(&args).output().unwrap();

    if !output.status.success() {
        let mut stdout: String =
            String::from_utf8(output.stdout).expect("Unable to parse java output");
        stdout.push_str(String::from_utf8(output.stderr).unwrap().as_str());

        panic!("Failed to run java {:?}: {}", args, stdout);
    }
}
