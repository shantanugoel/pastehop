use std::{env, path::PathBuf};

pub fn config_path() -> PathBuf {
    if let Ok(path) = env::var("PH_WEZTERM_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home)
            .join("wezterm")
            .join("wezterm.lua");
    }

    PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_owned()))
        .join(".config")
        .join("wezterm")
        .join("wezterm.lua")
}

pub fn render(binary_path: &str) -> String {
    include_str!("../../assets/wezterm/managed_block.lua").replace("__PH_BINARY__", binary_path)
}

pub fn default_config() -> &'static str {
    "local wezterm = require 'wezterm'\nlocal config = wezterm.config_builder()\n\nreturn config\n"
}
