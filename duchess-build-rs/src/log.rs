#[macro_export]
macro_rules! log {
    ($($tokens: tt)*) => {
        if std::env::var("DUCHESS_DEBUG").is_ok() {
            // print logging at warn level so it's easy to see (without intentionally failing the build)
            println!("cargo:warning={}", format!($($tokens)*))
        } else {
            // otherwise `eprintln` (this will be visible if the build script panics)
            eprintln!("{}", format!($($tokens)*))
        }
    }
}
