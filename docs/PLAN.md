# PasteHop Implementation Plan

Date: March 10, 2026
Status: Proposed MVP implementation plan
Primary implementation language: Rust

## Scope

The current MVP targets one terminal integration:

- `ph attach <paths...>`
- `ph attach --clipboard`
- `ph hook wezterm ...`
- `ph install wezterm`
- `ph uninstall wezterm`
- `ph doctor`
- `ph gc`

Explicitly deferred for now:

- terminal integrations beyond WezTerm
- nested SSH beyond one hop
- a background daemon
- a custom SSH library
- broad clipboard MIME support beyond practical image bytes and file lists

## Project Shape

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
assets/
  wezterm/
```

## Runtime Decisions

- Use the system `ssh` and `scp` commands.
- Keep terminal code thin and delegate decisions to Rust.
- Use remote file paths as the only compatibility primitive.
- Fail closed on ambiguous remote detection instead of guessing.

## Delivery Phases

### Phase 1: Core CLI and models

- command tree
- hook response model
- config load/save
- path formatting profiles
- typed errors

### Phase 2: Transport and attach

- remote staging path builder
- local file validation and size limits
- `ssh`/`scp` transport wrapper
- explicit host trust via config or CLI
- `ph attach`

### Phase 3: Clipboard image support

- `ph attach --clipboard`
- PNG materialization for raw clipboard images

### Phase 4: WezTerm integration

- `ph hook wezterm ...`
- WezTerm Lua installer template
- `ph install wezterm`
- `ph uninstall wezterm`

### Phase 5: Doctor and cleanup

- `ph doctor`
- `ph gc`
- best-effort cleanup on upload
- installation and troubleshooting docs

## UX Rules

1. Never break local paste behavior in WezTerm.
2. Never paste placeholder or partial garbage on failure.
3. Keep the success path silent: upload and paste only.
4. Prefer terse local error messaging over modal UX.
5. Keep `ph attach` as the reliable fallback.

## Definition Of Done

The MVP is done when:

- a user can copy an image locally, press `Ctrl+V` in WezTerm while in a one-hop SSH session, and get a remote file path pasted into the active pane
- `ph attach ./file.pdf` uploads and prints or pastes a remote path
- local sessions preserve normal paste behavior
- the project can be installed without any remote daemon or per-host setup beyond existing SSH access
