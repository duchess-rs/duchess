use std::{env, fs, io, path};

use duchess_build_rs::Configuration;

static JAR_URL: &str =
    "https://github.com/viperproject/viperserver/releases/download/v.23.01-release/viperserver.jar";

fn main() {
    let out_dir_string = env::var("OUT_DIR").unwrap();
    let out_dir = path::Path::new(&out_dir_string);
    let jar_path = out_dir.join("viper.jar");

    let jar_data = ureq::get(JAR_URL).call().unwrap();
    let mut jar_file = fs::File::create(jar_path.clone()).unwrap();
    io::copy(&mut jar_data.into_reader(), &mut jar_file).unwrap();

    duchess_build_rs::DuchessBuildRs::new()
        .with_configuration(Configuration::new().push_classpath(jar_path.display()))
        .execute()
        .unwrap();
}
