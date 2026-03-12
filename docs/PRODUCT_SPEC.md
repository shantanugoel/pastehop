# PasteHop Product Spec

Date: March 10, 2026
Status: Draft MVP spec
Primary implementation language: Rust

## Summary

`pastehop` is a local companion tool for terminal-based AI agents running on
remote machines.

Its core job is:

1. User presses the same image-paste key they already use in coding TUIs, typically `Ctrl+V`.
2. If the active pane is a remote SSH session and the local clipboard contains an image or file payload, PasteHop uploads that payload to the remote host.
3. PasteHop pastes the resulting remote path into the active terminal pane.
4. The remote TUI consumes that path as if the user had manually staged and pasted a file path.

## Primary User Story

1. User is in WezTerm on the local machine.
2. User SSHes to a Linux or macOS box.
3. User starts `codex` or `claude` inside that remote shell, optionally inside tmux or zellij.
4. User copies an image locally.
5. User presses `Ctrl+V`.
6. PasteHop uploads the image and pastes a remote path.

## MVP Scope

Included:

- WezTerm same-key integration
- clipboard image support
- explicit local file upload support via CLI
- one-hop SSH detection from the local terminal session
- remote upload to a staging directory
- path injection into the active pane
- `ph doctor`
- `ph gc`

Excluded for now:

- terminal integrations beyond WezTerm
- nested SSH beyond the first remote hop
- remote shell integration requirement
- bidirectional transfer
- full generic clipboard MIME bridge

## CLI Surface

- `ph attach <paths...>`
- `ph attach --clipboard`
- `ph hook wezterm ...`
- `ph install wezterm`
- `ph uninstall wezterm`
- `ph doctor`
- `ph gc`

Important flags:

- `--host <ssh-target>`
- `--remote-dir <path>`
- `--profile <plain-path|at-path|quoted-path>`
- `--print`
- `--dry-run`
- `--debug`

## Terminal Adapter Design

The core business logic lives in Rust. The WezTerm adapter is intentionally thin.

Common hook result:

```json
{
  "action": "inject_text | passthrough_key | noop | error",
  "text": "/remote/path/to/file.png",
  "message": "optional status"
}
```

WezTerm adapter:

- Installer adds Lua keybindings for `Ctrl+V`.
- Lua callback uses `wezterm.run_child_process(...)`.
- It inspects pane metadata such as foreground process and domain.
- On `inject_text`, it pastes the returned path.
- On `passthrough_key`, it replays the original key.

## Remote Target Resolution

Priority:

1. explicit `--host`
2. terminal-provided remote metadata
3. local foreground process inspection for `ssh`-style commands

If no remote host is resolved, PasteHop returns `passthrough-key`.

## Remote Staging

Default remote staging root:

- `~/.cache/pastehop/uploads/`

Per-upload path pattern:

- `~/.cache/pastehop/uploads/YYYY-MM-DD/<timestamp>-<kind>.<ext>`

## Security Model

- Only user-initiated keypresses or explicit CLI calls can trigger upload.
- PasteHop must never allow a remote process to silently read the local clipboard.
- PasteHop uses the user's existing SSH trust model.
- Unknown hosts must fail closed until explicitly trusted in config or via CLI.
- PasteHop fails closed when target resolution is ambiguous.

Default limits:

- max single file: 25 MB
- max total request: 100 MB
- max files per attach: 10

## Configuration

Local config file:

- `~/.config/pastehop/config.toml`

Key settings:

- default terminal profile
- enabled keybindings
- per-host allowlist
- per-host remote dir override
- size limits
- cleanup TTL

## Future Work

- optional remote shell integration for better host and cwd metadata
- additional terminal integrations once the WezTerm path is solid
- local file picker UI
- multi-file clipboard batches
- remote-to-local download symmetry
