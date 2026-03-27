use std::env;

#[derive(Debug, Clone)]
pub struct Environment {
    pub os: String,
    pub hostname: String,
}

impl Environment {
    /// Detects the current operating system and hostname.
    pub fn current() -> Self {
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        Self {
            os: env::consts::OS.to_string(),
            hostname,
        }
    }
}
