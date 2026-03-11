use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{errors::PasteHopError, profiles::PathProfile};

pub const DEFAULT_REMOTE_DIR: &str = "~/.cache/pastehop/uploads";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_terminal_profile: Option<String>,
    #[serde(default = "default_enabled_keybindings")]
    pub enabled_keybindings: Vec<String>,
    #[serde(default)]
    pub hosts: BTreeMap<String, HostConfig>,
    #[serde(default)]
    pub size_limits: SizeLimits,
    #[serde(default = "default_cleanup_ttl_hours")]
    pub cleanup_ttl_hours: u64,
    #[serde(default)]
    pub default_path_profile: PathProfile,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostConfig {
    #[serde(default)]
    pub allowed: bool,
    #[serde(default)]
    pub remote_dir: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SizeLimits {
    #[serde(default = "default_max_single_file_bytes")]
    pub max_single_file_bytes: u64,
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: u64,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_terminal_profile: None,
            enabled_keybindings: default_enabled_keybindings(),
            hosts: BTreeMap::new(),
            size_limits: SizeLimits::default(),
            cleanup_ttl_hours: default_cleanup_ttl_hours(),
            default_path_profile: PathProfile::PlainPath,
        }
    }
}

impl Default for SizeLimits {
    fn default() -> Self {
        Self {
            max_single_file_bytes: default_max_single_file_bytes(),
            max_total_bytes: default_max_total_bytes(),
            max_files: default_max_files(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Self {
        Self {
            path: default_config_path(),
        }
    }

    #[cfg(test)]
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Config, PasteHopError> {
        if !self.path.exists() {
            return Ok(Config::default());
        }

        let raw = fs::read_to_string(&self.path).map_err(|source| PasteHopError::ReadConfig {
            path: self.path.clone(),
            source,
        })?;

        toml::from_str(&raw).map_err(|source| PasteHopError::ParseConfig {
            path: self.path.clone(),
            source,
        })
    }

    pub fn save(&self, config: &Config) -> Result<(), PasteHopError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| PasteHopError::WriteConfig {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let raw = toml::to_string_pretty(config).map_err(|source| PasteHopError::EncodeConfig {
            path: self.path.clone(),
            source,
        })?;

        fs::write(&self.path, raw).map_err(|source| PasteHopError::WriteConfig {
            path: self.path.clone(),
            source,
        })
    }
}

pub fn default_config_path() -> PathBuf {
    if let Ok(path) = env::var("PH_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home)
            .join("pastehop")
            .join("config.toml");
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(home)
        .join(".config")
        .join("pastehop")
        .join("config.toml")
}

fn default_enabled_keybindings() -> Vec<String> {
    vec!["CTRL+V".to_owned()]
}

fn default_cleanup_ttl_hours() -> u64 {
    24
}

fn default_max_single_file_bytes() -> u64 {
    25 * 1024 * 1024
}

fn default_max_total_bytes() -> u64 {
    100 * 1024 * 1024
}

fn default_max_files() -> usize {
    10
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{Config, ConfigStore, DEFAULT_REMOTE_DIR};

    #[test]
    fn defaults_match_spec_basics() {
        let config = Config::default();

        assert_eq!(config.cleanup_ttl_hours, 24);
        assert_eq!(config.size_limits.max_single_file_bytes, 25 * 1024 * 1024);
        assert_eq!(config.size_limits.max_total_bytes, 100 * 1024 * 1024);
        assert_eq!(config.size_limits.max_files, 10);
        assert!(config.hosts.is_empty());
        assert_eq!(DEFAULT_REMOTE_DIR, "~/.cache/pastehop/uploads");
    }

    #[test]
    fn saves_and_loads_round_trip() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let path = temp_dir.path().join("pastehop").join("config.toml");
        let store = ConfigStore::with_path(&path);

        let mut config = Config::default();
        config.default_terminal_profile = Some("wezterm".to_owned());
        config.hosts.entry("devbox".to_owned()).or_default().allowed = true;

        store.save(&config).expect("config should save");
        let loaded = store.load().expect("config should load");

        assert_eq!(loaded, config);
        assert!(path.exists());
    }

    #[test]
    fn missing_config_returns_defaults() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let path = temp_dir.path().join("config.toml");
        let store = ConfigStore::with_path(path);

        let loaded = store.load().expect("missing config should not fail");

        assert_eq!(loaded, Config::default());
    }
}
