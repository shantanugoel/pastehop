# PasteHop

Terminal AI coding agents like Claude Code, Codex, and Pi can accept image
paths as input, but when you're SSH'd into a remote box your clipboard still
lives on your local machine. PasteHop uploads a local clipboard image or file
to the remote host over SSH and pastes the resulting remote path into the
active terminal pane. The CLI binary is `ph`.

## What It Does

- Preserves the common `Ctrl+V` flow in supported terminals (Currently WezTerm)
- Uploads clipboard images or explicit files to a remote staging directory over SSH
- Pastes a remote path that the agent can consume immediately
- Uses the system `ssh` and `scp`; no remote daemon or server-side install required

## Direct Integration Terminals

- WezTerm

## Install

```bash
cargo install --path .
```

Or build a local release binary:

```bash
cargo build --release
```

## Quick Start

### Option A: Auto-magic Ctrl+V (recommended)

This hooks into your terminal so that pressing `Ctrl+V` while in an SSH session
automatically uploads the clipboard image and pastes the remote path.

```bash
# 1. Check that your environment is ready (ssh, scp, clipboard access)
ph doctor

# 2. Install the WezTerm integration
ph install wezterm

# 3. That's it. Now:
#    - Copy an image on your local machine
#    - Focus a remote SSH session in WezTerm
#    - Press Ctrl+V
#    - PasteHop uploads the image and pastes the remote path
```

### Option B: Manual via the command line

Use `ph attach` directly when you want explicit control, are scripting, or
don't use the WezTerm integration.

```bash
# Upload a local file to a remote host and print the remote path
ph attach ./diagram.png --host user@devbox

# Upload whatever image is on your clipboard instead of a file
ph attach --clipboard --host user@devbox

# Dry-run to see what would happen without actually uploading
ph attach ./diagram.png --host user@devbox --dry-run
```

## Common Commands

### attach -- upload files or clipboard to a remote host

```bash
# Upload a single file
ph attach ./screenshot.png --host user@devbox

# Upload multiple files at once
ph attach ./fig1.png ./fig2.png --host user@devbox

# Upload the current clipboard image
ph attach --clipboard --host user@devbox

# Upload and also copy the resulting remote path to your clipboard
ph attach ./photo.jpg --host user@devbox --copy-path

# Preview what would be uploaded without actually doing it
ph attach ./diagram.png --host user@devbox --dry-run

# Use a specific path format
ph attach ./spec.pdf --host user@devbox --profile at-path
ph attach ./spec.pdf --host user@devbox --profile quoted-path

# Override the default remote upload directory
ph attach ./data.csv --host user@devbox --remote-dir /tmp/uploads
```

### install / uninstall -- set up or remove WezTerm Ctrl+V integration

```bash
ph install wezterm
ph uninstall wezterm
```

### doctor -- check that your environment is ready

```bash
ph doctor
```

### gc -- clean up expired remote uploads

```bash
ph gc --host user@devbox
ph gc --host user@devbox --dry-run
```

## Configuration

Configuration is optional. Editing the config file is only needed if you want to
pre-approve hosts, adjust size limits, or change the cleanup TTL.

See [`config.example.toml`](config.example.toml) for all available options.


The config path can also be set via `PH_CONFIG_PATH` or `XDG_CONFIG_HOME`.

### Paths

- Config: `~/.config/pastehop/config.toml`
- Remote uploads: `~/.cache/pastehop/uploads/`

## License

MIT. See [LICENSE](LICENSE).
