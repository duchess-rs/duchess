use std::{env, io, fs, path};
use zip_extensions::zip_extract;

static JAR_URL: &str = "https://github.com/viperproject/viperserver/releases/download/v.23.01-release/viperserver.jar";

fn main() {
    let out_dir_string = env::var("OUT_DIR").unwrap();
    let out_dir = path::Path::new(&out_dir_string);
    let jar_path = out_dir.join("viper.jar");
    let class_path = out_dir.join("classes");

    let jar_data = ureq::get(JAR_URL).call().unwrap();
    let mut jar_file = fs::File::create(jar_path.clone()).unwrap();
    io::copy(&mut jar_data.into_reader(), &mut jar_file).unwrap();
    zip_extract(&jar_path, &class_path).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", class_path.display());
    println!("cargo:rustc-env=CLASSPATH={}", class_path.display());
}
