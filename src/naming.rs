use std::path::Path;

use time::{OffsetDateTime, format_description::BorrowedFormatItem, macros::format_description};

static DATE_FORMAT: &[BorrowedFormatItem<'static>] = format_description!("[year]-[month]-[day]");
static TIME_FORMAT: &[BorrowedFormatItem<'static>] = format_description!("[hour][minute][second]");

pub fn build_remote_path(
    remote_root: &str,
    local_path: &Path,
    now: OffsetDateTime,
    kind: UploadKind,
) -> String {
    let date = now
        .format(DATE_FORMAT)
        .expect("date format should be valid");
    let time = now
        .format(TIME_FORMAT)
        .expect("time format should be valid");
    let ext = extension_for(local_path, kind);
    let stem = match kind {
        UploadKind::ClipboardImage => "clipboard".to_owned(),
        UploadKind::ExplicitFile => sanitize_file_stem(local_path),
    };

    format!(
        "{}/{}/{}-{}.{}",
        trim_trailing_slash(remote_root),
        date,
        time,
        stem,
        ext
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UploadKind {
    ClipboardImage,
    ExplicitFile,
}

fn trim_trailing_slash(path: &str) -> &str {
    path.trim_end_matches('/')
}

fn extension_for(local_path: &Path, kind: UploadKind) -> String {
    match kind {
        UploadKind::ClipboardImage => "png".to_owned(),
        UploadKind::ExplicitFile => local_path
            .extension()
            .and_then(|ext| ext.to_str())
            .filter(|ext| !ext.is_empty())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_else(|| "bin".to_owned()),
    }
}

fn sanitize_file_stem(local_path: &Path) -> String {
    let raw = local_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("file");
    let mut cleaned = String::with_capacity(raw.len());

    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            cleaned.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_whitespace() || ch == '.' {
            cleaned.push('-');
        }
    }

    let compacted = cleaned.trim_matches('-').replace("--", "-");
    if compacted.is_empty() {
        "file".to_owned()
    } else {
        compacted
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use time::OffsetDateTime;

    use super::{UploadKind, build_remote_path};

    #[test]
    fn preserves_explicit_file_extension() {
        let now = OffsetDateTime::from_unix_timestamp(1_762_892_645).expect("valid timestamp");
        let remote = build_remote_path(
            "~/.cache/pastehop/uploads",
            Path::new("/tmp/My Diagram.PDF"),
            now,
            UploadKind::ExplicitFile,
        );

        assert_eq!(
            remote,
            "~/.cache/pastehop/uploads/2025-11-11/202405-my-diagram.pdf"
        );
        assert!(!remote.contains(' '));
    }

    #[test]
    fn clipboard_images_use_png() {
        let now = OffsetDateTime::from_unix_timestamp(1_762_892_645).expect("valid timestamp");
        let remote = build_remote_path(
            "~/.cache/pastehop/uploads",
            Path::new("/tmp/ignored.bin"),
            now,
            UploadKind::ClipboardImage,
        );

        assert_eq!(
            remote,
            "~/.cache/pastehop/uploads/2025-11-11/202405-clipboard.png"
        );
    }
}
