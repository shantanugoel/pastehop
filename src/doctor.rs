use std::{env, path::PathBuf};

use crate::config::ConfigStore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctorCheck {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

pub fn run_doctor() -> Vec<DoctorCheck> {
    let store = ConfigStore::new();
    let config_path = store.path().to_path_buf();

    vec![
        check_tool("ssh"),
        check_tool("scp"),
        check_config_path(&config_path),
        check_clipboard(),
    ]
}

fn check_tool(tool: &'static str) -> DoctorCheck {
    match which::which(tool) {
        Ok(path) => DoctorCheck {
            name: tool,
            ok: true,
            detail: path.display().to_string(),
        },
        Err(error) => DoctorCheck {
            name: tool,
            ok: false,
            detail: error.to_string(),
        },
    }
}

fn check_config_path(path: &PathBuf) -> DoctorCheck {
    let parent = path
        .parent()
        .map(|dir| dir.display().to_string())
        .unwrap_or_else(|| ".".to_owned());
    DoctorCheck {
        name: "config",
        ok: true,
        detail: format!("path={} parent={}", path.display(), parent),
    }
}

fn check_clipboard() -> DoctorCheck {
    if env::var_os("PH_FAKE_CLIPBOARD_IMAGE").is_some()
        || env::var_os("PH_FAKE_CLIPBOARD_TEXT").is_some()
    {
        return DoctorCheck {
            name: "clipboard",
            ok: true,
            detail: "using fake clipboard override".to_owned(),
        };
    }

    match arboard::Clipboard::new() {
        Ok(_) => DoctorCheck {
            name: "clipboard",
            ok: true,
            detail: "clipboard handle available".to_owned(),
        },
        Err(error) => DoctorCheck {
            name: "clipboard",
            ok: false,
            detail: error.to_string(),
        },
    }
}
