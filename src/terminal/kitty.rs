use std::{
    env,
    path::{Path, PathBuf},
};

pub fn config_dir() -> PathBuf {
    if let Ok(path) = env::var("PH_KITTY_CONFIG_PATH") {
        return Path::new(&path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
    }

    if let Ok(path) = env::var("KITTY_CONFIG_DIRECTORY") {
        return PathBuf::from(path);
    }

    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home).join("kitty");
    }

    PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_owned()))
        .join(".config")
        .join("kitty")
}

pub fn config_path() -> PathBuf {
    if let Ok(path) = env::var("PH_KITTY_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    config_dir().join("kitty.conf")
}

pub fn kitten_path() -> PathBuf {
    config_dir().join(managed_kitten_name())
}

pub fn managed_kitten_name() -> &'static str {
    "pastehop.py"
}

pub fn managed_keys() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &["cmd+v", "ctrl+v", "ctrl+shift+v"]
    } else {
        &["ctrl+v", "ctrl+shift+v"]
    }
}

pub fn render_config() -> String {
    let mappings = managed_keys()
        .iter()
        .map(|key| format!("map {key} kitten {}", managed_kitten_name()))
        .collect::<Vec<_>>()
        .join("\n");

    include_str!("../../assets/kitty/managed_block.conf")
        .replace("__PH_KITTEN__", managed_kitten_name())
        .replace("__PH_MAPPINGS__", &mappings)
}

pub fn render_kitten(binary_path: &str) -> String {
    include_str!("../../assets/kitty/pastehop.py").replace("__PH_BINARY__", binary_path)
}

pub fn is_managed_kitten(contents: &str) -> bool {
    normalize_managed_kitten(contents).trim_end()
        == include_str!("../../assets/kitty/pastehop.py").trim_end()
}

fn normalize_managed_kitten(contents: &str) -> String {
    let mut normalized = String::new();
    for line in contents.lines() {
        if line.starts_with("PH_BINARY = r\"") && line.ends_with('"') {
            normalized.push_str("PH_BINARY = r\"__PH_BINARY__\"");
        } else {
            normalized.push_str(line);
        }
        normalized.push('\n');
    }
    normalized
}
