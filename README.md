# PasteHop

Paste local images and files into remote terminal agent sessions over SSH.

PasteHop uploads a clipboard image or local file to the current remote host and injects the resulting remote path back into the active terminal pane. The CLI binary is `ph`.

## What It Does

- Preserves the common `Ctrl+V` flow in supported terminals
- Uploads clipboard images or explicit files to a remote staging directory
- Pastes a remote path that tools like Codex or Claude Code can consume
- Uses the system `ssh` and `scp`; no remote daemon required

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

```bash
ph doctor
ph install wezterm
ph attach ./diagram.png --host devbox
ph attach --clipboard --host devbox --print
```

Typical same-key flow:

1. Copy an image locally.
2. Focus a remote SSH session in a supported terminal.
3. Press `Ctrl+V`.
4. PasteHop uploads the file and pastes the remote path.

## Common Commands

```bash
ph attach <paths...>
ph attach --clipboard
ph hook wezterm
ph hook kitty
ph hook iterm2
ph install <wezterm|kitty|iterm2>
ph uninstall <wezterm|kitty|iterm2>
ph doctor
ph gc --host <target>
```

## Paths

- Config: `~/.config/pastehop/config.toml`
- Remote uploads: `~/.cache/pastehop/uploads/`

## License

MIT. See [LICENSE](LICENSE).
