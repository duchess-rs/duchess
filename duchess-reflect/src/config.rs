use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
pub struct Configuration {
    data: Arc<Data>,
}

#[derive(Clone)]
struct Data {
    classpath: String,
    java_home: Option<PathBuf>,
}

impl Configuration {
    /// Create an empty configuration.
    /// The `Default::default`
    pub fn new() -> Self {
        Configuration {
            data: Arc::new(Data {
                classpath: String::new(),
                java_home: None,
            }),
        }
    }

    /// Load the configuration from environment variables.
    /// Invoked by `Self::default`.
    ///
    /// Reads the following environment variables (may grow over time):
    ///
    /// * `CLASSPATH`
    pub fn with_env(mut self) -> Self {
        let data = Arc::make_mut(&mut self.data);

        if let Ok(classpath) = std::env::var("CLASSPATH") {
            data.push_classpath(&classpath);
        }

        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            data.java_home = Some(PathBuf::from(java_home));
        }

        self
    }

    /// Extend the classpath with an additional entry
    pub fn push_classpath(mut self, path: impl ToString) -> Self {
        let data = Arc::make_mut(&mut self.data);
        data.push_classpath(&path.to_string());
        self
    }

    /// Override the classpath with an additional entry
    pub fn with_classpath(mut self, path: impl ToString) -> Self {
        let data = Arc::make_mut(&mut self.data);
        data.classpath = path.to_string();
        self
    }

    /// Extend the classpath with an additional entry
    pub fn with_java_home(mut self, path: impl AsRef<Path>) -> Self {
        let data = Arc::make_mut(&mut self.data);
        data.java_home = Some(path.as_ref().to_path_buf());
        self
    }

    /// Read current classpath configuration
    pub fn classpath(&self) -> Option<&str> {
        if self.data.classpath.is_empty() {
            None
        } else {
            Some(&self.data.classpath)
        }
    }

    /// Read current classpath configuration
    pub fn bin_path(&self, command: &str) -> PathBuf {
        if let Some(java_home) = &self.data.java_home {
            java_home.join("bin").join(command)
        } else {
            PathBuf::from(command)
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration::new().with_env()
    }
}

impl Data {
    fn push_classpath(&mut self, path: &str) {
        if !self.classpath.is_empty() {
            self.classpath.push(':');
        }
        self.classpath.push_str(path);
    }
}
