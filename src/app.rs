use anyhow::{Result, anyhow};
use time::OffsetDateTime;

use crate::{
    cli::{
        AttachArgs, Cli, Command, DoctorArgs, GcArgs, HookArgs, HookTerminal, InstallArgs,
        TerminalHookArgs, TrustArgs, UninstallArgs,
    },
    clipboard::{read_clipboard_image, write_clipboard_text},
    config::{Config, ConfigStore},
    doctor::run_doctor,
    errors::PasteHopError,
    gc::run_gc,
    hook::HookResponse,
    install::{install_terminal, uninstall_terminal},
    staging::{prepare_clipboard_upload, prepare_explicit_uploads},
    target::{HookTargetContext, resolve_attach_target, resolve_hook_target},
    transport::Transport,
};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Attach(args) => handle_attach(args),
        Command::Trust(args) => handle_trust(args),
        Command::Hook(args) => handle_hook(args),
        Command::Install(args) => handle_install(args),
        Command::Uninstall(args) => handle_uninstall(args),
        Command::Doctor(args) => handle_doctor(args),
        Command::Gc(args) => handle_gc(args),
    }
}

fn handle_attach(args: AttachArgs) -> Result<()> {
    if args.paths.is_empty() && !args.clipboard {
        return Err(wrap(PasteHopError::MissingAttachInput));
    }

    let store = ConfigStore::new();
    let config = store.load()?;
    let target = resolve_attach_target(args.host.as_deref(), args.remote_dir.as_deref(), &config)
        .map_err(wrap)?;
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let mut clipboard_upload = None;
    let uploads = if args.clipboard {
        let clipboard = read_clipboard_image().map_err(wrap)?;
        let upload = prepare_clipboard_upload(
            clipboard.file.path(),
            clipboard.size_bytes,
            &target,
            &config,
            args.profile,
            now,
        )
        .map_err(wrap)?;
        clipboard_upload = Some(clipboard);
        vec![upload]
    } else {
        prepare_explicit_uploads(&args.paths, &target, &config, args.profile, now).map_err(wrap)?
    };
    ensure_host_allowed(&config, &target.host).map_err(wrap)?;

    if args.dry_run {
        for upload in uploads {
            println!(
                "dry_run host={} local={} remote={}",
                target.host,
                upload.local_path.display(),
                upload.remote_path
            );
        }
        return Ok(());
    }

    let transport = Transport::discover().map_err(wrap)?;
    let cleanup_result =
        transport.cleanup_expired(&target.host, &target.remote_dir, config.cleanup_ttl_hours);

    for upload in &uploads {
        transport
            .ensure_remote_dir(&target.host, &upload.remote_path)
            .map_err(wrap)?;
        transport
            .upload_file(&target.host, &upload.local_path, &upload.remote_path)
            .map_err(wrap)?;
    }

    if args.debug {
        if let Err(error) = cleanup_result {
            eprintln!("cleanup warning: {error}");
        }
    }

    let formatted_paths: Vec<String> = uploads
        .into_iter()
        .map(|upload| upload.formatted_remote_path)
        .collect();

    if args.copy_path {
        write_clipboard_text(&formatted_paths.join("\n")).map_err(wrap)?;
    }

    for path in formatted_paths {
        println!("{path}");
    }

    drop(clipboard_upload);

    Ok(())
}

fn handle_trust(args: TrustArgs) -> Result<()> {
    let store = ConfigStore::new();
    let mut config = store.load()?;
    trust_host(&store, &mut config, &args.host, args.remote_dir.as_deref()).map_err(wrap)?;

    let message = if let Some(remote_dir) = args.remote_dir {
        format!(
            "trusted host '{}' with remote directory '{}'",
            args.host, remote_dir
        )
    } else {
        format!("trusted host '{}'", args.host)
    };
    println!("{message}");

    Ok(())
}

fn handle_hook(args: HookArgs) -> Result<()> {
    let store = ConfigStore::new();
    let mut config = store.load()?;
    let response = match args.terminal {
        HookTerminal::Wezterm(hook) => execute_hook(hook, &mut config),
    };

    let encoded = response
        .to_json()
        .map_err(|source| wrap(PasteHopError::HookSerialization(source)))?;
    println!("{encoded}");
    Ok(())
}

fn execute_hook(hook: TerminalHookArgs, config: &mut Config) -> HookResponse {
    let context = HookTargetContext {
        explicit_host: hook.host,
        remote_dir_override: hook.remote_dir,
        domain: hook.domain,
        foreground_process: hook.foreground_process,
    };
    let Some(target) = resolve_hook_target(&context, config) else {
        return HookResponse::passthrough_key();
    };

    if let Err(error) = ensure_host_allowed(config, &target.host) {
        return HookResponse::error(error.to_string());
    }

    let clipboard = match read_clipboard_image() {
        Ok(clipboard) => clipboard,
        Err(PasteHopError::ClipboardNotImage) => return HookResponse::passthrough_key(),
        Err(error) => return HookResponse::error(error.to_string()),
    };

    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let upload = match prepare_clipboard_upload(
        clipboard.file.path(),
        clipboard.size_bytes,
        &target,
        config,
        hook.profile,
        now,
    ) {
        Ok(upload) => upload,
        Err(error) => return HookResponse::error(error.to_string()),
    };

    let transport = match Transport::discover() {
        Ok(transport) => transport,
        Err(error) => return HookResponse::error(error.to_string()),
    };

    let _ = transport.cleanup_expired(&target.host, &target.remote_dir, config.cleanup_ttl_hours);

    if let Err(error) = transport.ensure_remote_dir(&target.host, &upload.remote_path) {
        return HookResponse::error(error.to_string());
    }
    if let Err(error) = transport.upload_file(&target.host, &upload.local_path, &upload.remote_path)
    {
        return HookResponse::error(error.to_string());
    }

    HookResponse::inject_text(upload.formatted_remote_path)
}

fn handle_install(args: InstallArgs) -> Result<()> {
    let message = install_terminal(args.terminal).map_err(wrap)?;
    println!("{message}");
    Ok(())
}

fn handle_uninstall(args: UninstallArgs) -> Result<()> {
    let message = uninstall_terminal(args.terminal).map_err(wrap)?;
    println!("{message}");
    Ok(())
}

fn handle_doctor(_args: DoctorArgs) -> Result<()> {
    for check in run_doctor() {
        let status = if check.ok { "ok" } else { "error" };
        println!("{} {} {}", status, check.name, check.detail);
    }
    Ok(())
}

fn handle_gc(args: GcArgs) -> Result<()> {
    let store = ConfigStore::new();
    let config = store.load()?;
    let target = resolve_attach_target(args.host.as_deref(), args.remote_dir.as_deref(), &config)
        .map_err(wrap)?;
    let transport = Transport::discover().map_err(wrap)?;
    let removed =
        run_gc(&transport, &target, config.cleanup_ttl_hours, args.dry_run).map_err(wrap)?;

    if removed.is_empty() {
        println!("no expired files found for {}", target.host);
    } else {
        for path in removed {
            println!("{path}");
        }
    }

    Ok(())
}

fn trust_host(
    store: &ConfigStore,
    config: &mut Config,
    host: &str,
    remote_dir: Option<&str>,
) -> Result<(), PasteHopError> {
    let entry = config.hosts.entry(host.to_owned()).or_default();
    entry.allowed = true;
    if let Some(remote_dir) = remote_dir {
        entry.remote_dir = Some(remote_dir.to_owned());
    }
    store.save(config)?;
    Ok(())
}

fn ensure_host_allowed(config: &Config, host: &str) -> Result<(), PasteHopError> {
    if config
        .hosts
        .get(host)
        .map(|entry| entry.allowed)
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(PasteHopError::HostDenied {
            host: host.to_owned(),
        })
    }
}

fn wrap(error: PasteHopError) -> anyhow::Error {
    let exit_code = error.exit_code();
    anyhow!(error).context(format!("exit_code={exit_code}"))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::config::ConfigStore;

    use super::{ensure_host_allowed, trust_host};

    #[test]
    fn unknown_hosts_are_rejected_without_prompting() {
        let error = ensure_host_allowed(&crate::config::Config::default(), "devbox")
            .expect_err("unknown host must be rejected");

        assert_eq!(
            error.to_string(),
            "host 'devbox' is not approved for uploads; run `ph trust --host devbox`"
        );
    }

    #[test]
    fn trust_host_marks_host_allowed_and_persists_remote_dir() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let store = ConfigStore::with_path(temp_dir.path().join("pastehop.toml"));
        let mut config = crate::config::Config::default();

        trust_host(&store, &mut config, "devbox", Some("/srv/uploads"))
            .expect("trust should succeed");

        assert!(
            ensure_host_allowed(&config, "devbox").is_ok(),
            "trusted host should be allowed"
        );

        let loaded = store.load().expect("saved config should load");
        let host = loaded
            .hosts
            .get("devbox")
            .expect("trusted host should be persisted");
        assert!(host.allowed);
        assert_eq!(host.remote_dir.as_deref(), Some("/srv/uploads"));
    }
}
