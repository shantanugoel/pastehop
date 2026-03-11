use std::{env, path::PathBuf};

pub fn script_path() -> PathBuf {
    if let Ok(path) = env::var("PH_ITERM2_SCRIPT_PATH") {
        return PathBuf::from(path);
    }

    PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_owned()))
        .join("Library")
        .join("Application Support")
        .join("iTerm2")
        .join("Scripts")
        .join("AutoLaunch")
        .join("pastehop.py")
}

pub fn render(binary_path: &str) -> String {
    include_str!("../../assets/iterm2/pastehop.py").replace("__PH_BINARY__", binary_path)
}
