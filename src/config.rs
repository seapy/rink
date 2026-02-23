use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_separator")]
    pub separator: String,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_ms: u64,
    #[serde(default = "default_sort")]
    pub default_sort: String,
    #[serde(default)]
    pub categories: HashMap<String, CategoryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryConfig {
    pub name: Option<String>,
    pub color: Option<String>,
}

fn default_separator() -> String {
    "-".to_string()
}

fn default_refresh_interval() -> u64 {
    2000
}

fn default_sort() -> String {
    "name".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            separator: default_separator(),
            refresh_interval_ms: default_refresh_interval(),
            default_sort: default_sort(),
            categories: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&PathBuf>) -> Self {
        let config_path = path.cloned().unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("rink")
                .join("config.toml")
        });

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => Config::default(),
            }
        } else {
            Config::default()
        }
    }
}
