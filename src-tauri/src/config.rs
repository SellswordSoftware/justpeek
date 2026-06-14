use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: String,
    pub theme: String,
    #[serde(default = "default_preferred_shortcut_os")]
    pub preferred_shortcut_os: String,
    #[serde(default = "default_shortcut_display_mode")]
    pub shortcut_display_mode: String,
    #[serde(default, alias = "shortcuts_dir")]
    pub references_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "CommandOrControl+Alt+Slash".to_string(),
            theme: "dark".to_string(),
            preferred_shortcut_os: default_preferred_shortcut_os(),
            shortcut_display_mode: default_shortcut_display_mode(),
            references_dir: None,
        }
    }
}

fn default_preferred_shortcut_os() -> String {
    "auto".to_string()
}

fn default_shortcut_display_mode() -> String {
    "current".to_string()
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("justpeek")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.yaml")
}

pub fn references_dir_path(config: &Config) -> PathBuf {
    config
        .references_dir
        .clone()
        .unwrap_or_else(|| config_dir().join("references"))
}

pub fn read_config() -> Config {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).expect("Failed to read config.yaml");
        serde_yaml::from_str(&content).unwrap_or_else(|_| Config::default())
    } else {
        let config = Config::default();
        save_config(&config);
        config
    }
}

pub fn save_config(config: &Config) {
    let dir = config_dir();
    fs::create_dir_all(&dir).expect("Failed to create config directory");
    let content = serde_yaml::to_string(config).expect("Failed to serialize config");
    fs::write(config_path(), content).expect("Failed to write config.yaml");
}

pub fn is_supported_preferred_shortcut_os(value: &str) -> bool {
    matches!(value.trim(), "auto" | "macos" | "windows" | "linux")
}

pub fn is_supported_shortcut_display_mode(value: &str) -> bool {
    matches!(value.trim(), "current" | "all")
}
