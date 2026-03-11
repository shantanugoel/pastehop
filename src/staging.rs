use std::{
    fs,
    path::{Path, PathBuf},
};

use time::OffsetDateTime;

use crate::{
    config::Config,
    errors::PasteHopError,
    naming::{UploadKind, build_remote_path},
    profiles::PathProfile,
    target::ResolvedTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedUpload {
    pub local_path: PathBuf,
    pub remote_path: String,
    pub formatted_remote_path: String,
    pub size_bytes: u64,
}

pub fn prepare_explicit_uploads(
    paths: &[PathBuf],
    target: &ResolvedTarget,
    config: &Config,
    profile: PathProfile,
    now: OffsetDateTime,
) -> Result<Vec<PreparedUpload>, PasteHopError> {
    if paths.is_empty() {
        return Err(PasteHopError::MissingAttachInput);
    }
    if paths.len() > config.size_limits.max_files {
        return Err(PasteHopError::TooManyFiles {
            limit: config.size_limits.max_files,
            actual: paths.len(),
        });
    }

    let mut uploads = Vec::with_capacity(paths.len());
    let mut total_size = 0_u64;

    for path in paths {
        let metadata = fs::metadata(path).map_err(|source| PasteHopError::ReadLocalPath {
            path: path.clone(),
            source,
        })?;
        if !metadata.is_file() {
            return Err(PasteHopError::InvalidLocalPath(path.clone()));
        }

        let size_bytes = metadata.len();
        if size_bytes > config.size_limits.max_single_file_bytes {
            return Err(PasteHopError::FileTooLarge {
                path: path.clone(),
                limit_bytes: config.size_limits.max_single_file_bytes,
                actual_bytes: size_bytes,
            });
        }

        total_size += size_bytes;
        if total_size > config.size_limits.max_total_bytes {
            return Err(PasteHopError::TotalSizeTooLarge {
                limit_bytes: config.size_limits.max_total_bytes,
                actual_bytes: total_size,
            });
        }

        let remote_path =
            build_remote_path(&target.remote_dir, path, now, UploadKind::ExplicitFile);
        uploads.push(PreparedUpload {
            local_path: path.clone(),
            formatted_remote_path: profile.format(&remote_path),
            remote_path,
            size_bytes,
        });
    }

    Ok(uploads)
}

pub fn prepare_clipboard_upload(
    local_path: &Path,
    size_bytes: u64,
    target: &ResolvedTarget,
    config: &Config,
    profile: PathProfile,
    now: OffsetDateTime,
) -> Result<PreparedUpload, PasteHopError> {
    if size_bytes > config.size_limits.max_single_file_bytes {
        return Err(PasteHopError::FileTooLarge {
            path: local_path.to_path_buf(),
            limit_bytes: config.size_limits.max_single_file_bytes,
            actual_bytes: size_bytes,
        });
    }

    let remote_path = build_remote_path(
        &target.remote_dir,
        local_path,
        now,
        UploadKind::ClipboardImage,
    );
    Ok(PreparedUpload {
        local_path: local_path.to_path_buf(),
        formatted_remote_path: profile.format(&remote_path),
        remote_path,
        size_bytes,
    })
}

pub fn remote_directory_for(path: &str) -> &Path {
    Path::new(path).parent().unwrap_or_else(|| Path::new(path))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::TempDir;
    use time::OffsetDateTime;

    use crate::{
        config::Config,
        profiles::PathProfile,
        target::{ResolutionSource, ResolvedTarget},
    };

    use super::{prepare_clipboard_upload, prepare_explicit_uploads, remote_directory_for};

    #[test]
    fn prepares_uploads_with_formatted_paths() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let local_path = temp_dir.path().join("Architecture Diagram.PDF");
        fs::write(&local_path, b"pdf").expect("local file should exist");

        let config = Config::default();
        let target = ResolvedTarget {
            host: "devbox".to_owned(),
            remote_dir: "/srv/uploads".to_owned(),
            source: ResolutionSource::ExplicitHost,
        };
        let now = OffsetDateTime::from_unix_timestamp(1_762_892_645).expect("valid timestamp");

        let uploads = prepare_explicit_uploads(
            &[local_path],
            &target,
            &config,
            PathProfile::QuotedPath,
            now,
        )
        .expect("uploads should prepare");

        assert_eq!(uploads.len(), 1);
        assert_eq!(
            uploads[0].formatted_remote_path,
            "\"/srv/uploads/2025-11-11/202405-architecture-diagram.pdf\""
        );
    }

    #[test]
    fn remote_directory_is_parent_of_remote_path() {
        let dir = remote_directory_for("/srv/uploads/2025-11-11/file.png");
        assert_eq!(dir, PathBuf::from("/srv/uploads/2025-11-11").as_path());
    }

    #[test]
    fn clipboard_uploads_use_clipboard_name_and_png() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let local_path = temp_dir.path().join("clipboard-source.tmp");
        fs::write(&local_path, b"png").expect("local file should exist");

        let config = Config::default();
        let target = ResolvedTarget {
            host: "devbox".to_owned(),
            remote_dir: "/srv/uploads".to_owned(),
            source: ResolutionSource::ExplicitHost,
        };
        let now = OffsetDateTime::from_unix_timestamp(1_762_892_645).expect("valid timestamp");

        let upload = prepare_clipboard_upload(
            &local_path,
            3,
            &target,
            &config,
            PathProfile::PlainPath,
            now,
        )
        .expect("clipboard upload should prepare");

        assert_eq!(
            upload.remote_path,
            "/srv/uploads/2025-11-11/202405-clipboard.png"
        );
    }
}
