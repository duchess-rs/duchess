fn main() {
    duchess_build_rs::DuchessBuildRs::new()
        .with_src_path("src/".into())
        .execute()
        .unwrap();
}
