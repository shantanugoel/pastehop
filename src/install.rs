use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{
    cli::SupportedTerminal,
    errors::PasteHopError,
    terminal::{iterm2, kitty, wezterm},
};

const WEZTERM_START: &str = "-- BEGIN PASTEHOP MANAGED BLOCK";
const WEZTERM_END: &str = "-- END PASTEHOP MANAGED BLOCK";
const KITTY_START: &str = "# BEGIN PASTEHOP MANAGED BLOCK";
const KITTY_END: &str = "# END PASTEHOP MANAGED BLOCK";

pub fn install_terminal(terminal: SupportedTerminal) -> Result<String, PasteHopError> {
    let binary_path = resolve_binary_path()?;

    match terminal {
        SupportedTerminal::Wezterm => {
            let path = wezterm::config_path();
            let rendered = wezterm::render(&binary_path);
            ensure_wezterm_scaffold(&path)?;
            install_managed_block(&path, &rendered, WEZTERM_START, WEZTERM_END, true)?;
            Ok(format!(
                "installed wezterm integration at {}",
                path.display()
            ))
        }
        SupportedTerminal::Kitty => {
            let path = kitty::config_path();
            let bridge_path = kitty::bridge_path();
            let rendered = kitty::render_config(&bridge_path.display().to_string());
            install_managed_block(&path, &rendered, KITTY_START, KITTY_END, false)?;
            write_support_file(&bridge_path, &kitty::render_bridge(&binary_path), true)?;
            Ok(format!("installed kitty integration at {}", path.display()))
        }
        SupportedTerminal::Iterm2 => {
            let path = iterm2::script_path();
            write_support_file(&path, &iterm2::render(&binary_path), true)?;
            Ok(format!(
                "installed iterm2 integration at {}",
                path.display()
            ))
        }
    }
}

pub fn uninstall_terminal(terminal: SupportedTerminal) -> Result<String, PasteHopError> {
    match terminal {
        SupportedTerminal::Wezterm => {
            let path = wezterm::config_path();
            remove_managed_block(&path, WEZTERM_START, WEZTERM_END)?;
            Ok(format!(
                "removed wezterm integration from {}",
                path.display()
            ))
        }
        SupportedTerminal::Kitty => {
            let path = kitty::config_path();
            remove_managed_block(&path, KITTY_START, KITTY_END)?;
            let bridge_path = kitty::bridge_path();
            if bridge_path.exists() {
                fs::remove_file(&bridge_path).map_err(|source| PasteHopError::InstallIo {
                    path: bridge_path.clone(),
                    source,
                })?;
            }
            Ok(format!("removed kitty integration from {}", path.display()))
        }
        SupportedTerminal::Iterm2 => {
            let path = iterm2::script_path();
            if path.exists() {
                fs::remove_file(&path).map_err(|source| PasteHopError::InstallIo {
                    path: path.clone(),
                    source,
                })?;
            }
            Ok(format!(
                "removed iterm2 integration from {}",
                path.display()
            ))
        }
    }
}

fn install_managed_block(
    path: &Path,
    block: &str,
    start_marker: &str,
    end_marker: &str,
    insert_before_return: bool,
) -> Result<(), PasteHopError> {
    let existing = read_optional(path)?;
    maybe_backup(path, &existing)?;
    let cleaned = strip_managed_block(&existing, start_marker, end_marker);
    let updated = if insert_before_return {
        insert_before_last_return(&cleaned, block)
    } else {
        append_block(&cleaned, block)
    };
    write_support_file(path, &updated, false)
}

fn remove_managed_block(
    path: &Path,
    start_marker: &str,
    end_marker: &str,
) -> Result<(), PasteHopError> {
    let existing = read_optional(path)?;
    let updated = strip_managed_block(&existing, start_marker, end_marker);
    write_support_file(path, updated.trim_end(), false)
}

fn resolve_binary_path() -> Result<String, PasteHopError> {
    if let Ok(path) = env::var("PH_BINARY_PATH") {
        return Ok(path);
    }

    std::env::current_exe()
        .map(|path| path.display().to_string())
        .map_err(|source| PasteHopError::InstallIo {
            path: PathBuf::from("current_exe"),
            source,
        })
}

fn ensure_wezterm_scaffold(path: &Path) -> Result<(), PasteHopError> {
    let existing = read_optional(path)?;
    if existing.trim().is_empty() {
        write_support_file(path, wezterm::default_config(), false)?;
    }
    Ok(())
}

fn read_optional(path: &Path) -> Result<String, PasteHopError> {
    if !path.exists() {
        return Ok(String::new());
    }

    fs::read_to_string(path).map_err(|source| PasteHopError::InstallIo {
        path: path.to_path_buf(),
        source,
    })
}

fn maybe_backup(path: &Path, existing: &str) -> Result<(), PasteHopError> {
    if existing.is_empty() {
        return Ok(());
    }

    let backup_path = backup_path(path);
    if backup_path.exists() {
        return Ok(());
    }

    fs::write(&backup_path, existing).map_err(|source| PasteHopError::InstallIo {
        path: backup_path,
        source,
    })
}

fn backup_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.ph.bak", path.display()))
}

fn write_support_file(path: &Path, contents: &str, executable: bool) -> Result<(), PasteHopError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| PasteHopError::InstallIo {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(path, format!("{}\n", contents.trim_end())).map_err(|source| {
        PasteHopError::InstallIo {
            path: path.to_path_buf(),
            source,
        }
    })?;

    if executable {
        let mut permissions = fs::metadata(path)
            .map_err(|source| PasteHopError::InstallIo {
                path: path.to_path_buf(),
                source,
            })?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|source| PasteHopError::InstallIo {
            path: path.to_path_buf(),
            source,
        })?;
    }

    Ok(())
}

fn append_block(existing: &str, block: &str) -> String {
    let trimmed = existing.trim_end();
    if trimmed.is_empty() {
        block.trim().to_owned()
    } else {
        format!("{trimmed}\n\n{}\n", block.trim())
    }
}

fn insert_before_last_return(existing: &str, block: &str) -> String {
    if let Some(index) = existing.rfind("\nreturn") {
        let (before, after) = existing.split_at(index + 1);
        format!(
            "{}\n\n{}\n{}",
            before.trim_end_matches('\n'),
            block.trim(),
            after
        )
    } else {
        append_block(existing, block)
    }
}

fn strip_managed_block(existing: &str, start_marker: &str, end_marker: &str) -> String {
    if let Some(start) = existing.find(start_marker) {
        if let Some(end_relative) = existing[start..].find(end_marker) {
            let end = start + end_relative + end_marker.len();
            let mut updated = String::new();
            updated.push_str(existing[..start].trim_end());
            if !updated.is_empty() && !existing[end..].trim().is_empty() {
                updated.push_str("\n\n");
            }
            updated.push_str(existing[end..].trim_start());
            return updated;
        }
    }

    existing.to_owned()
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use tempfile::TempDir;

    use crate::cli::SupportedTerminal;

    use super::{install_terminal, uninstall_terminal};

    #[test]
    fn kitty_install_is_idempotent_and_removable() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let config_path = temp_dir.path().join("kitty.conf");
        let bridge_path = temp_dir.path().join("ph-kitty-bridge.sh");

        unsafe {
            env::set_var("PH_BINARY_PATH", "/usr/local/bin/ph");
            env::set_var("PH_KITTY_CONFIG_PATH", &config_path);
            env::set_var("PH_KITTY_BRIDGE_PATH", &bridge_path);
        }

        install_terminal(SupportedTerminal::Kitty).expect("install should succeed");
        install_terminal(SupportedTerminal::Kitty).expect("reinstall should stay idempotent");

        let config = fs::read_to_string(&config_path).expect("config should exist");
        assert_eq!(config.matches("# BEGIN PASTEHOP MANAGED BLOCK").count(), 1);
        assert!(bridge_path.exists());

        uninstall_terminal(SupportedTerminal::Kitty).expect("uninstall should succeed");
        let removed = fs::read_to_string(&config_path).expect("config should still exist");
        assert!(!removed.contains("PASTEHOP MANAGED BLOCK"));
        assert!(!bridge_path.exists());

        unsafe {
            env::remove_var("PH_BINARY_PATH");
            env::remove_var("PH_KITTY_CONFIG_PATH");
            env::remove_var("PH_KITTY_BRIDGE_PATH");
        }
    }

    #[test]
    fn wezterm_install_creates_scaffold_for_empty_config() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let config_path = temp_dir.path().join("wezterm.lua");

        unsafe {
            env::set_var("PH_BINARY_PATH", "/usr/local/bin/ph");
            env::set_var("PH_WEZTERM_CONFIG_PATH", &config_path);
        }

        install_terminal(SupportedTerminal::Wezterm).expect("install should succeed");
        let config = fs::read_to_string(&config_path).expect("config should exist");
        assert!(config.contains("local wezterm = require 'wezterm'"));
        assert!(config.contains("return config"));
        assert!(config.contains("-- BEGIN PASTEHOP MANAGED BLOCK"));
        // Block should be before return
        let block_index = config
            .find("-- BEGIN PASTEHOP MANAGED BLOCK")
            .expect("block should exist");
        let return_index = config.find("return config").expect("return should exist");
        assert!(block_index < return_index);

        unsafe {
            env::remove_var("PH_BINARY_PATH");
            env::remove_var("PH_WEZTERM_CONFIG_PATH");
        }
    }

    #[test]
    fn wezterm_install_inserts_before_return() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let config_path = temp_dir.path().join("wezterm.lua");
        fs::write(&config_path, "local config = {}\nreturn config\n").expect("seed config");

        unsafe {
            env::set_var("PH_BINARY_PATH", "/usr/local/bin/ph");
            env::set_var("PH_WEZTERM_CONFIG_PATH", &config_path);
        }

        install_terminal(SupportedTerminal::Wezterm).expect("install should succeed");
        let config = fs::read_to_string(&config_path).expect("config should exist");
        let block_index = config
            .find("-- BEGIN PASTEHOP MANAGED BLOCK")
            .expect("block should exist");
        let return_index = config.find("return config").expect("return should exist");
        assert!(block_index < return_index);

        unsafe {
            env::remove_var("PH_BINARY_PATH");
            env::remove_var("PH_WEZTERM_CONFIG_PATH");
        }
    }
}
