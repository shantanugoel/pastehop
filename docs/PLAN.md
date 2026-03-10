# Image Paste Helper Implementation Plan

Date: March 10, 2026
Status: Proposed MVP implementation plan
Primary implementation language: Rust

## 1. Planning Goals

This plan converts `docs/PRODUCT_SPEC.md` into a lean implementation roadmap.

Primary constraints:

- keep the MVP simple to build and simple to maintain
- optimize for the highest-value UX first: remote image paste for coding agents
- avoid abstractions that are not needed for the first working release
- use Rust for the core logic and keep terminal-specific code thin

## 2. Simplicity Rules

These rules should guide implementation decisions:

1. Build one Rust binary crate first, not a multi-crate workspace.
2. Use the system `ssh` and `scp` commands instead of adding a custom SSH client layer.
3. Keep terminal adapters as small wrappers around a single Rust JSON hook interface.
4. Prefer blocking process execution with `std::process::Command` for the MVP. Do not add async unless a concrete need appears.
5. Use file paths as the only cross-terminal and cross-agent compatibility primitive.
6. Fail closed on ambiguous remote detection instead of guessing.
7. Ship Tier A terminal support incrementally rather than designing a generic plugin system.

## 3. MVP Scope To Build

The first release should include:

- `iph attach <paths...>`
- `iph attach --clipboard`
- `iph hook wezterm ...`
- `iph hook kitty ...`
- `iph hook iterm2 ...`
- `iph install <terminal>`
- `iph uninstall <terminal>`
- `iph doctor`
- `iph gc`
- clipboard image support on macOS and Linux
- explicit file upload support
- one-hop SSH target resolution
- remote staging under `~/.cache/image-paste-helper/uploads/`
- path injection for WezTerm, Kitty, and iTerm2

The first release should explicitly defer:

- nested SSH support beyond one hop
- a background daemon
- a custom SSH library
- terminal support outside WezTerm, Kitty, and iTerm2
- broad clipboard MIME support beyond image bytes and practical file lists
- a plugin system or separate Rust crates for each adapter

## 4. Proposed Rust Project Shape

Use a single binary crate with a small number of modules:

```text
src/
  main.rs
  cli.rs
  app.rs
  config.rs
  errors.rs
  clipboard.rs
  target.rs
  transport.rs
  staging.rs
  naming.rs
  profiles.rs
  hook.rs
  install.rs
  doctor.rs
  gc.rs
  terminal/
    mod.rs
    wezterm.rs
    kitty.rs
    iterm2.rs
assets/
  wezterm/
  kitty/
  iterm2/
```

Notes:

- `app.rs` should orchestrate use cases and keep CLI handlers thin.
- `terminal/*` should only parse adapter arguments and map to the shared hook response contract.
- terminal installer templates should live in `assets/`, not in generated Rust code.
- only split into multiple crates later if compile time or ownership boundaries become painful.

## 5. Lean Dependency Set

Start with the smallest practical dependency set:

- `clap` for CLI parsing
- `serde` and `serde_json` for hook responses and config serialization
- `toml` for config file parsing
- `anyhow` for top-level command errors
- `thiserror` for typed internal errors
- `arboard` for basic clipboard access
- `image` only if needed to normalize clipboard image bytes to PNG
- `tempfile` for staging local clipboard files
- `which` for checking required system tools

Avoid adding `tokio`, `sysinfo`, `sha2`, or a logging stack unless a real implementation gap appears.

## 6. Core Runtime Decisions

### Remote transport

- Use `ssh` for remote `mkdir`, `rm`, and related commands.
- Use `scp` for the initial upload path because it is simple and matches existing SSH config.
- Keep the transport module as a thin subprocess wrapper that returns clear success or failure results.

### Target resolution

Resolution order:

1. explicit `--host`
2. terminal-provided remote metadata
3. parsed local foreground process info for `ssh`-style commands

If none of the above is trustworthy, return `passthrough_key` for hooks or require `--host` for `attach`.

### Clipboard handling

- fully support raw image clipboard paste
- support CLI file paths
- support clipboard file lists only where the platform exposes them cleanly
- always convert raw clipboard images into PNG before upload

### Config

Use one local config file:

- `~/.config/image-paste-helper/config.toml`

Keep the schema small:

- per-host allowlist
- per-host remote dir override
- size limits
- cleanup TTL
- default path profile

## 7. Delivery Phases

### Phase 1: CLI skeleton and shared models

Deliverables:

- Rust binary with `clap` command tree
- shared hook response model with `inject_text`, `passthrough_key`, `noop`, and `error`
- config load/save helpers
- path formatting profiles
- error types and exit-code conventions

Acceptance criteria:

- `iph --help` shows the full planned command surface
- hook commands can return valid JSON without side effects
- config file is created on first write and read correctly after restart

### Phase 2: Transport and explicit attach

Deliverables:

- remote staging path builder
- local file validation and size-limit checks
- `ssh`/`scp` transport wrapper
- first-use host confirmation prompt
- `iph attach <paths...>`
- `iph attach --print`
- `iph attach --dry-run`

Acceptance criteria:

- uploading a local PNG to a reachable SSH host returns a valid remote path
- generated names follow the spec pattern and contain no spaces
- ambiguous or missing target resolution produces a clear error instead of a guessed host
- cleanup failures never block upload success

Why this phase comes early:

- it proves the hardest product primitive first: reliable upload to the correct remote host
- it creates a usable fallback UX before same-key integration lands

### Phase 3: Clipboard image support

Deliverables:

- `iph attach --clipboard`
- clipboard image detection on macOS and Linux
- temp-file materialization as PNG
- best-effort clipboard file-list support where straightforward

Acceptance criteria:

- copying an image locally and running `iph attach --clipboard --print` yields a remote path
- text-only clipboard content fails cleanly for `attach --clipboard`
- Linux file-list support is optional and does not block image support

### Phase 4: WezTerm integration

Deliverables:

- `iph hook wezterm ...`
- WezTerm Lua installer template
- `iph install wezterm`
- `iph uninstall wezterm`
- same-key `Ctrl+V` interception with passthrough on local sessions

Acceptance criteria:

- local session + image clipboard => original `Ctrl+V` reaches the app unchanged
- remote SSH session + image clipboard => uploaded file path is pasted into the pane
- remote SSH session + text clipboard => original `Ctrl+V` is replayed

Why WezTerm first:

- strongest metadata and scripting support
- best path to validating the hook contract without adapter complexity

### Phase 5: Kitty integration

Deliverables:

- `iph hook kitty ...`
- Kitty config managed block
- text injection via Kitty remote control
- passthrough-key replay support

Acceptance criteria:

- same decision matrix as WezTerm works in Kitty
- installer is idempotent and removable
- hook failures do not inject partial text

### Phase 6: iTerm2 integration

Deliverables:

- minimal Python bridge script under managed install
- `iph hook iterm2 ...`
- `iph install iterm2`
- `iph uninstall iterm2`

Acceptance criteria:

- same-key remote image paste works in iTerm2 on macOS
- local passthrough behavior matches the spec
- the Python bridge remains thin and delegates decision logic to Rust

### Phase 7: Doctor, cleanup, and release polish

Deliverables:

- `iph doctor` for environment checks
- `iph gc` for explicit cleanup
- best-effort cleanup on upload
- concise stderr messages and optional terminal-native notifications where trivial
- installation and troubleshooting docs

Acceptance criteria:

- `iph doctor` can detect missing `ssh`, `scp`, clipboard access issues, and config paths
- `iph gc --host <target>` removes expired staged files without affecting fresh uploads
- debug output is useful without requiring a telemetry system

## 8. UX Rules During Implementation

The implementation should preserve these behaviors across all phases:

1. Never break local paste behavior in supported terminals.
2. Never paste placeholder or partial garbage on failure.
3. Keep success-path interaction silent: upload and paste only.
4. Prefer terse local error messaging over modal UX.
5. Make `iph attach` the reliable fallback whenever same-key integration cannot decide safely.

## 9. Testing Strategy

Keep testing practical and cheap to maintain.

### Automated tests

- unit tests for naming, config parsing, size limits, and path formatting
- unit tests for hook decision logic
- integration tests using fake `ssh` and `scp` executables in `PATH` to verify subprocess behavior without real network dependencies
- installer tests that verify managed block insertion and removal in temp config files

### Manual smoke tests

- macOS + WezTerm + remote Linux
- macOS + iTerm2 + remote Linux
- Linux + WezTerm + remote Linux
- Linux + Kitty + remote Linux
- remote shell directly
- remote shell inside tmux
- remote shell inside zellij

Do not build PTY-heavy end-to-end automation in the MVP unless manual testing proves too expensive.

## 10. Recommended Build Order

Build in this order:

1. Phase 1
2. Phase 2
3. Phase 3
4. Phase 4
5. Phase 7 partial for `doctor`
6. Phase 5
7. Phase 6
8. Phase 7 remainder

This gets a working product into users' hands early:

- `attach` works before same-key support is complete
- WezTerm validates the terminal-hook architecture before the other adapters
- doctor/install/gc arrive before broadening support too far

## 11. Explicit Non-Engineering Guardrails

To avoid over-engineering, do not add these in the MVP:

- no internal plugin system
- no adapter trait hierarchy beyond a small shared hook contract
- no background process or agent
- no remote daemon
- no generalized event bus
- no database
- no attempt to detect every terminal and session shape automatically

## 12. Definition Of Done For MVP

The MVP is done when all of the following are true:

- a user can copy an image locally, press `Ctrl+V` in WezTerm, Kitty, or iTerm2 while in a one-hop SSH session, and get a remote file path pasted into the active pane
- a user can run `iph attach ./file.pdf` and get the remote path pasted or printed
- local sessions preserve normal paste behavior
- the project can be installed without any remote daemon or per-host remote setup beyond existing SSH access
- the codebase remains a small Rust CLI with thin adapter assets and clear test coverage around the risky logic
