use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PathProfile {
    #[default]
    PlainPath,
    AtPath,
    QuotedPath,
}

impl PathProfile {
    pub fn format(self, path: &str) -> String {
        match self {
            Self::PlainPath => path.to_owned(),
            Self::AtPath => format!("@{path}"),
            Self::QuotedPath => format!("\"{path}\""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PathProfile;

    #[test]
    fn formats_plain_path() {
        assert_eq!(
            PathProfile::PlainPath.format("/tmp/file.png"),
            "/tmp/file.png"
        );
    }

    #[test]
    fn formats_at_path() {
        assert_eq!(
            PathProfile::AtPath.format("/tmp/file.png"),
            "@/tmp/file.png"
        );
    }

    #[test]
    fn formats_quoted_path() {
        assert_eq!(
            PathProfile::QuotedPath.format("/tmp/file.png"),
            "\"/tmp/file.png\""
        );
    }
}
