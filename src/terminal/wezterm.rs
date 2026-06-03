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

    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(home)
        .join(".config")
        .join("wezterm")
        .join("wezterm.lua")
}

pub fn render(binary_path: &str) -> String {
    let escaped_path = if cfg!(windows) {
        binary_path.replace('\\', "\\\\")
    } else {
        binary_path.to_owned()
    };
    include_str!("../../assets/wezterm/managed_block.lua").replace("__PH_BINARY__", &escaped_path)
}

pub fn default_config() -> &'static str {
    "local wezterm = require 'wezterm'\nlocal config = wezterm.config_builder()\n\nreturn config\n"
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn render_escapes_backslashes_on_windows() {
        if cfg!(windows) {
            let rendered = render(r"C:\Program Files\pastehop\ph.exe");
            assert!(rendered.contains(r"C:\\Program Files\\pastehop\\ph.exe"));
            assert!(!rendered.contains(r"C:\Program Files\pastehop\ph.exe"));
        } else {
            let rendered = render("/usr/local/bin/ph");
            assert!(rendered.contains("/usr/local/bin/ph"));
        }
    }
}
