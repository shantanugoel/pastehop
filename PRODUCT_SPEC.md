# Image Paste Helper Product Spec

Date: March 10, 2026
Status: Draft MVP spec
Primary implementation language: Rust

## 1. Summary

`image-paste-helper` is a local companion tool for terminal-based AI agents running on remote machines.

Its core job is:

1. User presses the same image-paste key they already use in coding TUIs, typically `Ctrl+V`.
2. If the active pane is a remote SSH session and the local clipboard contains an image or file payload, the helper uploads that payload to the remote host.
3. The helper pastes the resulting remote path into the active terminal pane.
4. The remote TUI consumes that path as if the user had manually staged and pasted a file path.

The first release is optimized for remote coding-agent image paste, not generic clipboard sync.

## 2. Problem

Local coding TUIs such as Codex CLI and Claude Code can accept image paste because the app can read the local OS clipboard or local file paths. In a remote session over SSH, tmux, or zellij, the app is running on the remote host and cannot read the local clipboard.

The missing capability is not "remote clipboard access" in general. The missing capability is:

- detect a local non-text clipboard payload at paste time
- stage it onto the current remote host
- inject a remote file reference into the active pane with minimal workflow change

## 3. Product Goals

### Must have

- Fix remote image paste for coding TUIs on macOS and Linux local machines.
- Preserve the user's existing `Ctrl+V` image-paste workflow in supported terminals.
- Require no remote daemon for the MVP.
- Work when the remote TUI is inside plain SSH, SSH plus tmux, or SSH plus zellij on the same remote host.
- Use remote file paths as the compatibility layer so it works across multiple agents.

### Should have

- Support arbitrary file upload from explicit local paths, not just clipboard images.
- Support clipboard file-list payloads where the local OS exposes them.
- Keep install under a few minutes and avoid per-host manual setup for the common case.

### Nice to have

- Better nested-SSH support.
- Terminal-specific premium integrations.
- Remote cleanup policies and host-specific UX polish.

## 4. Non-Goals

- Full clipboard synchronization between local and remote machines.
- General remote desktop or GUI clipboard forwarding.
- A universal same-key solution for every terminal emulator on day one.
- Modifying Codex, Claude Code, Copilot CLI, or tmux/zellij themselves.
- Windows support in the MVP.

## 5. Primary User Stories

### Story A: Remote Codex image paste

1. User is in Kitty, WezTerm, or iTerm2 on the local machine.
2. User SSHes to a Linux or macOS box.
3. User starts `codex` inside that remote shell, optionally inside tmux or zellij.
4. User copies an image locally.
5. User presses `Ctrl+V`.
6. The helper uploads the image to the remote host and pastes a remote path like `~/.cache/image-paste-helper/2026-03-10/clipboard-193455.png`.

### Story B: Remote Claude Code image paste

Same as Story A, but the pasted artifact is a remote image path that Claude Code can attach/read.

### Story C: General attach flow

1. User has a local file they want to share with a remote agent.
2. User runs `iph attach path/to/file.pdf`.
3. The helper uploads the file to the remote host and pastes the resulting remote path into the current pane.

## 6. Product Positioning

The product has two modes.

### Mode 1: Agent Paste

This is the MVP and the main product.

- Triggered by the app-level paste keys used by coding TUIs, primarily `Ctrl+V` and optionally `Alt+V`.
- Only intercepts in supported terminals.
- If the active pane is not remote, the original key is passed through unchanged.
- If the clipboard is not a supported non-text payload, the original key is passed through unchanged.

### Mode 2: Generic Attach

This broadens the product beyond coding agents.

- Triggered by `iph attach ...` or optional terminal-specific bindings.
- Accepts local file paths and later clipboard file/image payloads.
- Uploads to remote and pastes remote path text.

This mode does not need exact workflow parity to be useful.

## 7. Core Product Decision

The compatibility primitive is a remote file path, not inline clipboard bytes.

Reasons:

- It works across agents without needing app changes.
- It works for images and arbitrary files.
- It works through tmux and zellij because they only need to pass text.
- It keeps the local helper independent from any one agent protocol.

## 8. Support Tiers

### Tier A: Same-key MVP support

- WezTerm
- Kitty
- iTerm2 on macOS

These terminals have enough local scripting or control APIs to intercept a key, run a local helper, and inject text back into the active pane.

### Tier B: Assisted support

- Ghostty
- Alacritty
- GNOME Terminal
- Terminal.app

These terminals do not currently offer a clean cross-platform same-key integration path for this use case. They will use `iph attach` first, and may get terminal-specific integrations later.

### Tier C: Premium future integrations

- Kitty native clipboard MIME/paste-event support
- Better nested-SSH metadata
- Terminal-specific drag/drop or command-palette flows

## 9. MVP Scope

### Included

- Local OS support: macOS and Linux
- Remote OS support: macOS and Linux
- One-hop SSH detection from the local terminal session
- Remote upload to a staging directory
- Path injection into the active pane
- Clipboard image support
- Explicit local file upload support via CLI
- Clipboard file-list support on platforms where it is straightforward
- tmux/zellij compatibility when they are running on the detected remote host

### Excluded

- Nested SSH beyond the first remote hop
- Mosh-first support
- Remote shell integration requirement
- Bidirectional transfer
- Full generic clipboard MIME bridge

## 10. UX Requirements

### UX 1: No surprise in local sessions

If the active pane is local, the product must not degrade local coding-agent paste behavior.

For `Ctrl+V` interception this means:

- local session + clipboard image/file => pass `Ctrl+V` through to the app unchanged
- local session + unsupported clipboard => pass `Ctrl+V` through unchanged

### UX 2: No visible transport ceremony

For a successful remote image paste in a supported terminal, the user should experience:

- copy image
- press `Ctrl+V`
- see a remote path appear in the prompt/editor

No modal picker should appear in the success path.

### UX 3: Predictable failure handling

If remote detection fails or upload fails:

- do not paste partial garbage
- show a short local notification where the terminal supports it
- print a terse status line in stderr or terminal logs
- preserve the ability to retry with `iph attach`

### UX 4: Zero remote install for common case

The first successful upload to a host should work as long as the user can already SSH to that host.

## 11. Detailed User Flows

### Flow A: Successful remote image paste

1. Terminal intercepts `Ctrl+V`.
2. Adapter calls `iph hook ...` with terminal context.
3. Helper checks clipboard payload types.
4. Helper resolves current remote target.
5. Helper materializes clipboard image as a temp PNG.
6. Helper uploads it to remote staging dir.
7. Helper returns an inject plan containing remote path text.
8. Terminal adapter injects text as paste-style input.

### Flow B: Local coding-agent image paste

1. Terminal intercepts `Ctrl+V`.
2. Helper sees no remote target.
3. Helper returns `passthrough-key`.
4. Terminal adapter replays `Ctrl+V` to the app.

### Flow C: Clipboard contains plain text

1. Terminal intercepts `Ctrl+V`.
2. Helper sees text-only clipboard or unsupported clipboard payload.
3. Helper returns `passthrough-key`.
4. Terminal adapter replays `Ctrl+V`.

This is acceptable because the primary workflow here is coding-agent image paste, not terminal text paste.

### Flow D: Explicit attach

1. User runs `iph attach ./diagram.png`.
2. Helper resolves current remote target from the active terminal context or explicit `--host`.
3. Helper uploads file.
4. Helper pastes remote path into the active pane or prints it to stdout if `--print` is set.

## 12. CLI Surface

Binary name: `iph`

### Commands

- `iph attach <paths...>`
- `iph attach --clipboard`
- `iph hook wezterm ...`
- `iph hook kitty ...`
- `iph hook iterm2 ...`
- `iph doctor`
- `iph install wezterm`
- `iph install kitty`
- `iph install iterm2`
- `iph uninstall <terminal>`
- `iph gc`

### Important flags

- `--host <ssh-target>`
- `--remote-dir <path>`
- `--profile <plain-path|at-path|quoted-path>`
- `--print`
- `--dry-run`
- `--debug`

## 13. Terminal Adapter Design

The core business logic lives in Rust. Terminal adapters are intentionally thin.

### Common contract

Each adapter invokes the helper and receives a small JSON result:

```json
{
  "action": "inject_text | passthrough_key | noop | error",
  "text": "/remote/path/to/file.png",
  "message": "optional status"
}
```

### WezTerm adapter

Implementation:

- Installer adds Lua keybindings for `Ctrl+V` and optionally `Alt+V`.
- Lua callback uses `wezterm.run_child_process(...)`.
- It can inspect pane data via `pane:get_foreground_process_info()`, `pane:get_current_working_dir()`, and `pane:get_domain_name()`.
- On `inject_text`, use `pane:send_paste(text)`.
- On `passthrough_key`, replay the original key to the pane.

### Kitty adapter

Implementation:

- Installer adds key mappings to `kitty.conf`.
- Mapping launches local `iph hook kitty ...`.
- The hook uses Kitty remote control to inject text into the active window.
- On `passthrough_key`, it sends the original key to the active window.

### iTerm2 adapter

Implementation:

- Installer drops a small Python API script under the iTerm2 scripts directory.
- User binds `Ctrl+V` to `Invoke Script Function`.
- The Python bridge calls `iph hook iterm2 ...`.
- On `inject_text`, the script uses `Session.async_send_text(...)`.
- On `passthrough_key`, the script replays the original key to the active session.

## 14. Remote Target Resolution

Target resolution priority:

1. Terminal-provided remote domain or session metadata.
2. Local foreground process inspection for `ssh`, `ssh -J`, `wezterm ssh`, `kitten ssh`, or similar.
3. Explicit `--host`.

If no remote host is resolved, the helper returns `passthrough-key`.

### MVP assumptions

- The common case is one local SSH client connected to one remote host.
- tmux and zellij are on that remote host.
- The current effective remote path can be the staging path rather than the remote cwd.

### Known limitation

If the user SSHes from the first remote host to a second host, the local machine cannot reliably infer that second hop in the MVP.

## 15. Remote Staging

Default remote staging root:

- `~/.cache/image-paste-helper/uploads/`

Per-upload path pattern:

- `~/.cache/image-paste-helper/uploads/YYYY-MM-DD/<timestamp>-<kind>.<ext>`

Examples:

- `2026-03-10/193455-clipboard.png`
- `2026-03-10/193501-architecture.pdf`

Rules:

- no spaces in generated names
- preserve original extension when uploading explicit files
- encode raw clipboard images as PNG

## 16. Transport Design

### MVP transport

Use the user's existing OpenSSH setup from the local machine.

Preferred implementation path:

- use Rust process spawning around `ssh` and `scp`, or `ssh` plus streamed stdin
- reuse existing host aliases, ProxyJump, keys, agent, certificates, and config

Required remote operations:

1. `mkdir -p <staging-dir>`
2. upload bytes
3. optional best-effort cleanup of old files

### Why not a custom SSH library

- OpenSSH compatibility is the real requirement, not protocol novelty.
- Users already rely on complex SSH config.
- Wrapping the system `ssh` keeps the product compatible with that config.

### Future transport

- Kitty TTY-native transfer
- Shared control-master caching
- Better batch transfer for multi-file uploads

## 17. Clipboard Handling

### Clipboard payloads to support in MVP

- raw image bytes
- explicit file paths passed on CLI
- clipboard file lists where platform support is practical

### Local macOS

Support:

- raw image clipboard
- file URLs from pasteboard

### Local Linux

Support:

- raw image clipboard
- best-effort file list support where clipboard exposes URI lists cleanly

### Important design choice

The MVP must fully solve image clipboard paste even if clipboard file-list support remains partial on Linux.

## 18. Path Formatting Profiles

Default profile: `plain-path`

Examples:

- `/home/user/.cache/image-paste-helper/uploads/2026-03-10/193455-clipboard.png`

Optional profiles:

- `at-path` => `@/home/user/.../file.png`
- `quoted-path` => `"/home/user/.../file.png"`

The default should be `plain-path` because it is the least opinionated and works with path-aware agents.

## 19. Security Model

### Principles

- Only user-initiated keypresses or explicit CLI calls can trigger upload.
- The helper must never allow a remote process to silently read the local clipboard.
- The helper must use the user's existing SSH trust model.
- The helper must fail closed when target resolution is ambiguous.

### First-use host confirmation

On first upload to a host, prompt locally with:

- resolved target
- remote staging dir
- size of upload

Persist allow or deny per host alias in local config.

### Limits

Default limits:

- max single file: 25 MB
- max total request: 100 MB
- max files per attach: 10

## 20. Configuration

Local config file:

- `~/.config/image-paste-helper/config.toml`

Key settings:

- default terminal profile
- enabled keybindings
- per-host allowlist
- per-host remote dir override
- size limits
- cleanup TTL

## 21. Cleanup Policy

Default remote TTL:

- 24 hours

Cleanup behavior:

- best-effort cleanup on each upload for files older than TTL
- `iph gc --host <target>` for explicit cleanup

Cleanup failures must never block a successful upload.

## 22. Rust Architecture

### Crates

- `iph-core`
- `iph-cli`
- `iph-terminal-wezterm` support code generated by installer
- `iph-terminal-iterm2` support script generated by installer

### Major modules

- `clipboard`
- `target_resolver`
- `staging`
- `transport`
- `naming`
- `profiles`
- `config`
- `doctor`
- `terminal_protocol`

### Candidate dependencies

- `tokio`
- `clap`
- `serde`
- `serde_json`
- `toml`
- `anyhow`
- `thiserror`
- `tracing`
- `arboard`
- `image`
- `tempfile`
- `sha2`
- `which`
- `sysinfo`

Platform-specific clipboard support may require small macOS-specific modules beyond `arboard`.

## 23. Installer Design

`iph install <terminal>` should:

1. detect config file location
2. add a clearly marked managed block
3. back up the file once
4. print exact changes made
5. avoid duplicate installs

Managed blocks should be removable by `iph uninstall <terminal>`.

## 24. Observability

The product should not require telemetry.

Local logs:

- default off
- debug logs in `~/.cache/image-paste-helper/logs/`
- `iph doctor` prints environment and capability checks

## 25. Performance Targets

- helper startup to action decision: under 100 ms on warm local path
- 5 MB image upload on a normal SSH link: under 2 seconds
- path injection after successful upload: immediate

## 26. Test Matrix

### Local OS

- macOS
- Linux Wayland
- Linux X11

### Remote OS

- Ubuntu-like Linux
- macOS

### Session shapes

- local terminal -> ssh -> codex
- local terminal -> ssh -> tmux -> codex
- local terminal -> ssh -> zellij -> codex
- local terminal -> ssh -> claude

### Clipboard payloads

- raw image
- text only
- file path on CLI
- clipboard file list where supported

## 27. Risks and Trade-Offs

### Risk: same-key support is terminal-specific

Trade-off:

- accept Tier A support first
- provide `iph attach` everywhere else

### Risk: nested SSH is ambiguous

Trade-off:

- explicitly document one-hop SSH MVP
- add shell-integration metadata later if needed

### Risk: Linux clipboard file lists are inconsistent

Trade-off:

- guarantee image clipboard support first
- make file-list clipboard best-effort on Linux

### Risk: path-only attach is less magical than true image paste

Trade-off:

- path attach is the broadest compatible primitive across agents

## 28. Future Work

- optional remote shell integration for better host and cwd metadata
- Kitty paste-event and arbitrary MIME support
- Ghostty or other terminal integrations as APIs mature
- local file picker UI
- upload progress UI in terminals that support overlays
- multi-file clipboard batches
- remote-to-local download symmetry

## 29. Recommendation

Build the MVP around remote coding-agent image paste first:

1. Rust core plus CLI
2. WezTerm adapter
3. Kitty adapter
4. iTerm2 adapter
5. explicit `iph attach` for generic files

That gets the highest-value workflow working quickly without betting on unsupported terminal behavior.
