use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{errors::PasteHopError, staging::remote_directory_for};

#[derive(Clone, Debug)]
pub struct Transport {
    ssh_bin: PathBuf,
    scp_bin: PathBuf,
}

impl Transport {
    pub fn discover() -> Result<Self, PasteHopError> {
        Ok(Self {
            ssh_bin: discover_bin("PH_SSH_BIN", "ssh")?,
            scp_bin: discover_bin("PH_SCP_BIN", "scp")?,
        })
    }

    pub fn ensure_remote_dir(&self, host: &str, remote_path: &str) -> Result<(), PasteHopError> {
        let remote_dir = remote_directory_for(remote_path);
        let script = format!(
            "mkdir -p -- {}",
            shell_quote(&remote_dir.display().to_string())
        );
        self.run_ssh(host, &script).map(|_| ())
    }

    pub fn upload_file(
        &self,
        host: &str,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<(), PasteHopError> {
        let destination = format!("{host}:{remote_path}");
        let status = Command::new(&self.scp_bin)
            .arg(local_path)
            .arg(&destination)
            .status()
            .map_err(|source| PasteHopError::SpawnTransport {
                command: self.scp_bin.clone(),
                source,
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(PasteHopError::TransportFailed {
                command: self.scp_bin.clone(),
                code: status.code(),
            })
        }
    }

    pub fn cleanup_expired(
        &self,
        host: &str,
        remote_root: &str,
        ttl_hours: u64,
    ) -> Result<(), PasteHopError> {
        let minutes = ttl_hours.saturating_mul(60);
        let script = format!(
            "find {} -type f -mmin +{} -delete 2>/dev/null || true",
            shell_quote(remote_root),
            minutes
        );
        self.run_ssh(host, &script).map(|_| ())
    }

    pub fn gc_expired(
        &self,
        host: &str,
        remote_root: &str,
        ttl_hours: u64,
        dry_run: bool,
    ) -> Result<Vec<String>, PasteHopError> {
        let minutes = ttl_hours.saturating_mul(60);
        let script = if dry_run {
            format!(
                "find {} -type f -mmin +{} -print 2>/dev/null || true",
                shell_quote(remote_root),
                minutes
            )
        } else {
            format!(
                "find {} -type f -mmin +{} -print -delete 2>/dev/null || true",
                shell_quote(remote_root),
                minutes
            )
        };

        let output = self.run_ssh(host, &script)?;
        Ok(output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(str::to_owned)
            .collect())
    }

    fn run_ssh(&self, host: &str, script: &str) -> Result<String, PasteHopError> {
        let output = Command::new(&self.ssh_bin)
            .arg(host)
            .arg("sh")
            .arg("-lc")
            .arg(script)
            .output()
            .map_err(|source| PasteHopError::SpawnTransport {
                command: self.ssh_bin.clone(),
                source,
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            Err(PasteHopError::TransportFailed {
                command: self.ssh_bin.clone(),
                code: output.status.code(),
            })
        }
    }
}

fn discover_bin(env_name: &str, default_name: &str) -> Result<PathBuf, PasteHopError> {
    if let Some(value) = env::var_os(env_name).filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(value));
    }

    which::which(default_name).map_err(move |_| PasteHopError::MissingTool(default_name.to_owned()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use std::{env, fs, os::unix::fs::PermissionsExt, path::Path};

    use tempfile::TempDir;

    use super::Transport;

    #[test]
    fn transport_uses_fake_ssh_and_scp_from_env() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let bin_dir = temp_dir.path().join("bin");
        let logs_dir = temp_dir.path().join("logs");
        fs::create_dir_all(&bin_dir).expect("bin dir should exist");
        fs::create_dir_all(&logs_dir).expect("logs dir should exist");

        let ssh_path = bin_dir.join("ssh");
        let scp_path = bin_dir.join("scp");
        let upload_source = temp_dir.path().join("diagram.png");
        fs::write(&upload_source, b"png").expect("upload source should exist");

        write_script(
            &ssh_path,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {}/ssh.log\nexit 0\n",
                logs_dir.display()
            ),
        );
        write_script(
            &scp_path,
            &format!(
                "#!/bin/sh\necho \"$@\" >> {}/scp.log\nexit 0\n",
                logs_dir.display()
            ),
        );

        unsafe {
            env::set_var("PH_SSH_BIN", &ssh_path);
            env::set_var("PH_SCP_BIN", &scp_path);
        }

        let transport = Transport::discover().expect("transport should discover");
        transport
            .ensure_remote_dir("devbox", "/srv/uploads/2025-11-11/file.png")
            .expect("mkdir should succeed");
        transport
            .upload_file("devbox", &upload_source, "/srv/uploads/2025-11-11/file.png")
            .expect("scp should succeed");
        transport
            .cleanup_expired("devbox", "/srv/uploads", 24)
            .expect("cleanup should succeed");
        let listed = transport
            .gc_expired("devbox", "/srv/uploads", 24, true)
            .expect("gc dry run should succeed");

        let ssh_log =
            fs::read_to_string(logs_dir.join("ssh.log")).expect("ssh log should be readable");
        let scp_log =
            fs::read_to_string(logs_dir.join("scp.log")).expect("scp log should be readable");

        assert!(ssh_log.contains("devbox sh -lc mkdir -p -- '/srv/uploads/2025-11-11'"));
        assert!(ssh_log.contains("find '/srv/uploads' -type f -mmin +1440 -delete"));
        assert!(listed.is_empty());
        assert!(scp_log.contains("diagram.png devbox:/srv/uploads/2025-11-11/file.png"));

        unsafe {
            env::remove_var("PH_SSH_BIN");
            env::remove_var("PH_SCP_BIN");
        }
    }

    fn write_script(path: &Path, content: &str) {
        fs::write(path, content).expect("script should be written");
        let mut permissions = fs::metadata(path)
            .expect("script metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("script should be executable");
    }
}
