#[derive(Debug, Clone)]
pub struct Config {
    pub separator: String,
    pub refresh_interval_ms: u64,
    pub default_sort: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            separator: "-".to_string(),
            refresh_interval_ms: 2000,
            default_sort: "name".to_string(),
        }
    }
}
