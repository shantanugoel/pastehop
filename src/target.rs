use crate::{
    config::{Config, DEFAULT_REMOTE_DIR},
    errors::PasteHopError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedTarget {
    pub host: String,
    pub remote_dir: String,
    pub source: ResolutionSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionSource {
    ExplicitHost,
    TerminalDomain,
    ForegroundProcess,
}

#[derive(Clone, Debug, Default)]
pub struct HookTargetContext {
    pub explicit_host: Option<String>,
    pub remote_dir_override: Option<String>,
    pub domain: Option<String>,
    pub foreground_process: Option<String>,
}

pub fn resolve_attach_target(
    explicit_host: Option<&str>,
    remote_dir_override: Option<&str>,
    config: &Config,
) -> Result<ResolvedTarget, PasteHopError> {
    let Some(host) = explicit_host.filter(|value| !value.trim().is_empty()) else {
        return Err(PasteHopError::MissingTarget);
    };

    Ok(ResolvedTarget {
        host: host.to_owned(),
        remote_dir: resolve_remote_dir(host, remote_dir_override, config),
        source: ResolutionSource::ExplicitHost,
    })
}

pub fn resolve_hook_target(context: &HookTargetContext, config: &Config) -> Option<ResolvedTarget> {
    if let Some(host) = context
        .explicit_host
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return Some(ResolvedTarget {
            host: host.to_owned(),
            remote_dir: resolve_remote_dir(host, context.remote_dir_override.as_deref(), config),
            source: ResolutionSource::ExplicitHost,
        });
    }

    if let Some(host) = context.domain.as_deref().and_then(parse_terminal_domain) {
        return Some(ResolvedTarget {
            host: host.clone(),
            remote_dir: resolve_remote_dir(&host, context.remote_dir_override.as_deref(), config),
            source: ResolutionSource::TerminalDomain,
        });
    }

    if let Some(target) = context
        .foreground_process
        .as_deref()
        .and_then(parse_foreground_process)
        .map(|host| ResolvedTarget {
            remote_dir: resolve_remote_dir(&host, context.remote_dir_override.as_deref(), config),
            host,
            source: ResolutionSource::ForegroundProcess,
        })
    {
        return Some(target);
    }

    None
}

pub fn parse_terminal_domain(domain: &str) -> Option<String> {
    domain
        .strip_prefix("SSH:")
        .or_else(|| domain.strip_prefix("ssh:"))
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
}

pub fn parse_foreground_process(process: &str) -> Option<String> {
    let tokens: Vec<&str> = process.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let basename = tokens[0].rsplit('/').next().unwrap_or(tokens[0]);
    match basename {
        "ssh" => parse_ssh_target(&tokens[1..]),
        "wezterm" if tokens.get(1) == Some(&"ssh") => parse_simple_host(&tokens[2..]),
        _ => None,
    }
}

fn parse_simple_host(tokens: &[&str]) -> Option<String> {
    tokens
        .iter()
        .copied()
        .find(|token| !token.starts_with('-'))
        .map(str::to_owned)
}

fn parse_ssh_target(tokens: &[&str]) -> Option<String> {
    let mut idx = 0;
    while idx < tokens.len() {
        let token = tokens[idx];
        if token == "--" {
            return tokens.get(idx + 1).copied().map(str::to_owned);
        }

        if requires_value(token) {
            idx += 2;
            continue;
        }

        if token.starts_with('-') {
            idx += 1;
            continue;
        }

        return Some(token.to_owned());
    }

    None
}

fn requires_value(token: &str) -> bool {
    matches!(
        token,
        "-B" | "-b"
            | "-c"
            | "-D"
            | "-E"
            | "-e"
            | "-F"
            | "-I"
            | "-i"
            | "-J"
            | "-L"
            | "-l"
            | "-m"
            | "-O"
            | "-o"
            | "-p"
            | "-Q"
            | "-R"
            | "-S"
            | "-W"
            | "-w"
    )
}

fn resolve_remote_dir(host: &str, override_dir: Option<&str>, config: &Config) -> String {
    override_dir
        .map(str::to_owned)
        .or_else(|| {
            config
                .hosts
                .get(host)
                .and_then(|host_config| host_config.remote_dir.clone())
        })
        .unwrap_or_else(|| DEFAULT_REMOTE_DIR.to_owned())
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, HostConfig};

    use super::{
        HookTargetContext, ResolutionSource, parse_foreground_process, parse_terminal_domain,
        resolve_attach_target, resolve_hook_target,
    };

    #[test]
    fn parse_domain_extracts_ssh_target() {
        assert_eq!(
            parse_terminal_domain("SSH:devbox"),
            Some("devbox".to_owned())
        );
        assert_eq!(parse_terminal_domain("local"), None);
    }

    #[test]
    fn parse_foreground_process_supports_ssh_and_wrappers() {
        assert_eq!(
            parse_foreground_process("ssh -J jump devbox"),
            Some("devbox".to_owned())
        );
        assert_eq!(
            parse_foreground_process("wezterm ssh devbox"),
            Some("devbox".to_owned())
        );
        assert_eq!(parse_foreground_process("bash"), None);
        assert_eq!(
            parse_foreground_process("/usr/bin/ssh devbox"),
            Some("devbox".to_owned())
        );
        assert_eq!(
            parse_foreground_process("/usr/bin/ssh -J jump user@devbox"),
            Some("user@devbox".to_owned())
        );
    }

    #[test]
    fn attach_resolution_requires_explicit_host() {
        let config = Config::default();
        let error = resolve_attach_target(None, None, &config).expect_err("missing host must fail");

        assert_eq!(
            error.to_string(),
            "could not resolve a remote target; pass --host"
        );
    }

    #[test]
    fn hook_resolution_prefers_explicit_host() {
        let mut config = Config::default();
        config.hosts.insert(
            "devbox".to_owned(),
            HostConfig {
                allowed: true,
                remote_dir: Some("/srv/uploads".to_owned()),
            },
        );
        let context = HookTargetContext {
            explicit_host: Some("devbox".to_owned()),
            remote_dir_override: None,
            domain: Some("SSH:other".to_owned()),
            foreground_process: Some("ssh third".to_owned()),
        };

        let target = resolve_hook_target(&context, &config).expect("target should resolve");
        assert_eq!(target.host, "devbox");
        assert_eq!(target.remote_dir, "/srv/uploads");
        assert_eq!(target.source, ResolutionSource::ExplicitHost);
    }
}
