use std::{env, path::PathBuf};

pub fn config_path() -> PathBuf {
    if let Ok(path) = env::var("PH_KITTY_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home).join("kitty").join("kitty.conf");
    }

    PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_owned()))
        .join(".config")
        .join("kitty")
        .join("kitty.conf")
}

pub fn bridge_path() -> PathBuf {
    if let Ok(path) = env::var("PH_KITTY_BRIDGE_PATH") {
        return PathBuf::from(path);
    }

    config_path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("ph-kitty-bridge.sh")
}

pub fn render_config(bridge_path: &str) -> String {
    include_str!("../../assets/kitty/managed_block.conf")
        .replace("__PH_KITTY_BRIDGE__", bridge_path)
}

pub fn render_bridge(binary_path: &str) -> String {
    include_str!("../../assets/kitty/bridge.sh").replace("__PH_BINARY__", binary_path)
}
