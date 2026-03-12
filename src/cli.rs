use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::profiles::PathProfile;

#[derive(Debug, Parser)]
#[command(
    name = "ph",
    version,
    about = "PasteHop for remote coding-agent sessions"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Upload local files or clipboard content and emit a remote path
    Attach(AttachArgs),
    /// Approve a remote host for uploads
    Trust(TrustArgs),
    /// Terminal hook entry points
    Hook(HookArgs),
    /// Install terminal integration
    Install(InstallArgs),
    /// Remove terminal integration
    Uninstall(UninstallArgs),
    /// Validate local environment and configuration
    Doctor(DoctorArgs),
    /// Remove expired staged uploads
    Gc(GcArgs),
}

#[derive(Debug, Args)]
pub struct AttachArgs {
    /// Local files to upload
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Upload supported content from the local clipboard
    #[arg(long, conflicts_with = "paths")]
    pub clipboard: bool,

    /// Explicit SSH target such as user@host or host alias
    #[arg(long)]
    pub host: Option<String>,

    /// Override the remote upload directory
    #[arg(long)]
    pub remote_dir: Option<String>,

    /// Formatting profile for emitted paths
    #[arg(long, value_enum, default_value_t = PathProfile::PlainPath)]
    pub profile: PathProfile,

    /// Print the remote path instead of trying terminal injection
    #[arg(long)]
    pub print: bool,

    /// Copy emitted remote path output to the local clipboard
    #[arg(long)]
    pub copy_path: bool,

    /// Resolve and describe the upload without performing it
    #[arg(long)]
    pub dry_run: bool,

    /// Enable debug-oriented output
    #[arg(long)]
    pub debug: bool,
}

#[derive(Debug, Args)]
pub struct TrustArgs {
    /// Explicit SSH target such as user@host or host alias
    #[arg(long)]
    pub host: String,

    /// Persist a default remote upload directory for this host
    #[arg(long)]
    pub remote_dir: Option<String>,
}

#[derive(Debug, Args)]
pub struct HookArgs {
    #[command(subcommand)]
    pub terminal: HookTerminal,
}

#[derive(Debug, Subcommand)]
pub enum HookTerminal {
    Wezterm(TerminalHookArgs),
}

#[derive(Debug, Args, Clone)]
pub struct TerminalHookArgs {
    /// Explicit SSH target override
    #[arg(long)]
    pub host: Option<String>,

    /// Explicit remote directory override
    #[arg(long)]
    pub remote_dir: Option<String>,

    /// Formatting profile for injected paths
    #[arg(long, value_enum, default_value_t = PathProfile::PlainPath)]
    pub profile: PathProfile,

    /// Original key for passthrough replay
    #[arg(long, default_value = "CTRL+V")]
    pub key: String,

    /// Domain or session metadata supplied by the terminal
    #[arg(long)]
    pub domain: Option<String>,

    /// Foreground process command line
    #[arg(long)]
    pub foreground_process: Option<String>,

    /// Current working directory as reported by the terminal
    #[arg(long)]
    pub cwd: Option<String>,

    /// Enable debug-oriented output
    #[arg(long)]
    pub debug: bool,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    #[arg(value_enum)]
    pub terminal: SupportedTerminal,
}

#[derive(Debug, Args)]
pub struct UninstallArgs {
    #[arg(value_enum)]
    pub terminal: SupportedTerminal,
}

#[derive(Debug, Args, Default)]
pub struct DoctorArgs {
    /// Enable debug-oriented output
    #[arg(long)]
    pub debug: bool,
}

#[derive(Debug, Args)]
pub struct GcArgs {
    /// Explicit SSH target to clean
    #[arg(long)]
    pub host: Option<String>,

    /// Override the remote upload directory
    #[arg(long)]
    pub remote_dir: Option<String>,

    /// Report intended actions without deleting files
    #[arg(long)]
    pub dry_run: bool,

    /// Enable debug-oriented output
    #[arg(long)]
    pub debug: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum SupportedTerminal {
    Wezterm,
}
