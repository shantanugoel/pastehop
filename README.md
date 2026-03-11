# PasteHop

Terminal AI coding agents like Claude Code, Codex, and Pi can accept image
paths as input -- but when you're SSH'd into a remote box, there's no way to
paste a local screenshot or diagram into that session. Your clipboard lives on
your machine; the agent runs on the server.

PasteHop bridges that gap. It uploads a clipboard image or local file to the
remote host over SSH and injects the resulting remote path straight into the
active terminal pane. To the agent it looks like you just typed a file path.
The CLI binary is `ph`.

## What It Does

- Preserves the common `Ctrl+V` flow in supported terminals
- Uploads clipboard images or explicit files to a remote staging directory over SSH
- Pastes a remote path that the agent can consume immediately
- Uses the system `ssh` and `scp`; no remote daemon or server-side install required

## Supported Terminals

- WezTerm
- Kitty
- iTerm2

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

# 2. Install the integration for your terminal (one-time setup)
ph install wezterm    # or: ph install kitty / ph install iterm2

# 3. That's it. Now:
#    - Copy an image on your local machine
#    - Focus a remote SSH session in your terminal
#    - Press Ctrl+V
#    - PasteHop uploads the image and pastes the remote path
```

### Option B: Manual via the command line

Use `ph attach` directly when you want explicit control, are scripting, or
don't use a supported terminal.

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

# Use a specific path format (useful for tools that expect @path or "path")
ph attach ./spec.pdf --host user@devbox --profile at-path
ph attach ./spec.pdf --host user@devbox --profile quoted-path

# Override the default remote upload directory
ph attach ./data.csv --host user@devbox --remote-dir /tmp/uploads
```

### install / uninstall -- set up or remove terminal Ctrl+V integration

```bash
# Install the hook for your terminal (pick one)
ph install wezterm
ph install kitty
ph install iterm2

# Remove it later if you no longer need it
ph uninstall wezterm
```

### doctor -- check that your environment is ready

```bash
# Verifies ssh, scp, clipboard access, and config location
ph doctor
```

### gc -- clean up expired remote uploads

```bash
# Remove uploads older than the configured TTL (default 24h)
ph gc --host user@devbox

# See what would be removed without deleting anything
ph gc --host user@devbox --dry-run
```

## Configuration

Configuration is **optional**. PasteHop works out of the box with sensible
defaults. A config file is only needed if you want to skip typing `--host`
every time, pre-approve hosts, adjust size limits, or change the cleanup TTL.

See [`config.example.toml`](config.example.toml) for all available options
with descriptions and defaults.

To use a config, copy the example and edit it:

```bash
cp config.example.toml ~/.config/pastehop/config.toml
```

The config path can also be set via `PH_CONFIG_PATH` or `XDG_CONFIG_HOME`.

### Paths

- Config: `~/.config/pastehop/config.toml`
- Remote uploads: `~/.cache/pastehop/uploads/`

## License

MIT. See [LICENSE](LICENSE).
