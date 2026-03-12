# Kitty Integration Plan

Date: March 12, 2026
Status: Feasibility spike complete

## Summary

Kitty is a viable direct-integration target for PasteHop.

The feasibility spike validated the key primitives we need:

- a keybinding can launch a custom kitten from `kitty.conf`
- the kitten can run with `no_ui=True`, so there is no visible overlay
- the kitten receives `target_window_id` and a `Boss` handle
- the kitten can inject text into the originating window
- the kitten can invoke Kitty's native `paste_from_clipboard` action for passthrough
- Kitty exposes window metadata through `kitten @ ls`, including cwd and command line

The main caveat versus WezTerm is remote target discovery. Kitty does not appear to expose WezTerm-style SSH domain metadata, so v1 should rely on:

- explicit `--host`
- parsing the foreground `ssh ...` command line from Kitty window metadata
- docs that recommend `kitten ssh` and/or standard `ssh` usage for best results

## Spike Verdict

Ship a Kitty adapter.

I would treat the remaining work as implementation and live-window verification, not as a feasibility blocker.

## Capability Findings

### 1. Bind a key to a custom local helper

Kitty documents custom kittens stored in the Kitty config directory and mapped from `kitty.conf`, for example:

```conf
map ctrl+k kitten mykitten.py
```

This is sufficient for a PasteHop-managed keybinding.

### 2. Run without opening a UI

Kitty documents `@result_handler(no_ui=True)` for kittens that only script Kitty and do not need a TUI overlay.

That matches PasteHop's design. The keybinding can trigger a local helper and return immediately without an intermediate prompt.

### 3. Target the original window

Kitty passes `target_window_id` into `handle_result(args, answer, target_window_id, boss)`.

This gives us a stable way to operate on the originating window instead of the currently active one. That matters because the PasteHop kitten must always inject or paste back into the window where the key was pressed.

### 4. Inject text

Kitty supports both of these paths:

- direct window API: `w.paste_text(answer)`
- remote control API: `boss.call_remote_control(..., ('send-text', '--match=id:...', ...))`

For PasteHop, the remote-control path is preferable because it is documented and lets us use `--bracketed-paste=auto`.

### 5. Preserve native paste on passthrough

This was the main blocker to prove.

Kitty exposes the `paste_from_clipboard` mappable action, and the remote control API exposes `kitten @ action ACTION [ARGS...]`.

That means a kitten can fall back to Kitty's own paste behavior instead of faking it. The intended call shape is:

```python
boss.call_remote_control(
    window,
    ("action", f"--match=id:{window.id}", "paste_from_clipboard"),
)
```

This is the strongest signal that Kitty can preserve normal paste behavior in local sessions and text-clipboard sessions.

### 6. Read window metadata

Kitty documents `kitten @ ls` returning JSON for windows, including:

- id
- current working directory
- process id
- command line
- environment

This is enough for the Kitty adapter to query metadata for the target window and pass a normalized foreground process string into `ph hook kitty ...`.

### 7. Default paste key differs from WezTerm

Kitty's documented default paste shortcut is:

- `ctrl+shift+v`
- `cmd+v` on macOS

The original recommendation was to stick to Kitty-native defaults. In practice, PasteHop's workflow is clearer if Linux installs both `ctrl+v` and `ctrl+shift+v`, while macOS installs `cmd+v`, `ctrl+v`, and `ctrl+shift+v`.

## Recommended Adapter Design

### Rust side

Add Kitty as both:

- `SupportedTerminal::Kitty`
- `HookTerminal::Kitty`

The Rust hook execution path can stay shared with WezTerm by reusing `TerminalHookArgs` and `execute_hook()`.

### Kitty side

Install two managed artifacts:

1. a managed Python kitten in the Kitty config directory, for example:
   `~/.config/kitty/pastehop.py`
2. a managed block in `kitty.conf`

The keybinding should call the kitten, not shell out directly from the config.

### Metadata flow

The kitten should use documented remote control output, not undocumented Kitty internals, for session metadata.

Recommended flow:

1. keybinding launches `pastehop.py`
2. `handle_result(..., target_window_id, boss)` resolves the target window
3. the kitten obtains metadata for that window via `kitten @ ls --match id:<target_window_id>`
4. the kitten extracts `cmdline` and optionally `cwd`
5. the kitten runs local `ph hook kitty ...`
6. the kitten parses the JSON response and maps it to Kitty actions

This keeps terminal-specific logic thin and preserves the existing PasteHop architecture.

### Response mapping

Map `ph hook kitty` responses like this:

- `inject_text`:
  `kitten @ send-text --match id:<window-id> --bracketed-paste=auto <text>`
- `passthrough_key`:
  `kitten @ action --match id:<window-id> paste_from_clipboard`
- `noop`:
  return silently
- `error`:
  prefer a small local notification via `kitten notify`, otherwise log to stderr

For `inject_text`, `send-text --bracketed-paste=auto` is preferable to raw typing.

## Implementation Plan

### Phase 1: CLI and module plumbing

Files:

- `src/cli.rs`
- `src/app.rs`
- `src/terminal/mod.rs`
- `src/terminal/kitty.rs`

Tasks:

- add `kitty` to terminal enums
- dispatch `ph hook kitty`
- add Kitty config path helpers and rendered asset helpers

### Phase 2: Installer and uninstaller

Files:

- `src/install.rs`
- `assets/kitty/pastehop.py`

Tasks:

- write the managed kitten file
- insert a managed block into `kitty.conf`
- create backups like the WezTerm installer does
- remove the block cleanly on uninstall
- remove the managed kitten file on uninstall if it still matches generated contents

### Phase 3: Kitty adapter behavior

Files:

- `assets/kitty/pastehop.py`
- optionally `src/target.rs`

Tasks:

- query Kitty metadata for the source window
- normalize command line into the existing hook model
- call `ph hook kitty`
- inject text or call native paste action based on hook response
- surface errors with minimal interruption

### Phase 4: Tests and docs

Files:

- `src/install.rs`
- `src/cli.rs`
- `README.md`
- `config.example.toml`
- `docs/PLAN.md`
- `docs/PRODUCT_SPEC.md`

Tasks:

- add installer tests for Kitty config scaffolding and managed block behavior
- add enum and dispatch coverage for Kitty
- document Kitty-native default keybindings
- document limitations around target detection

### Phase 5: Live manual validation

Run these scenarios in a real Kitty session:

- local shell + image clipboard -> normal paste behavior if no remote target resolves
- local shell + text clipboard -> native paste
- SSH session + trusted host + image clipboard -> remote path injection
- SSH session + untrusted host -> error notification, no upload
- SSH into tmux or zellij -> still works when the visible local foreground process remains `ssh`

## Risks And Open Questions

### 1. End-to-end GUI validation is still pending

This spike validated the documented and installed Kitty surfaces, but it did not run a live keypress in an interactive Kitty window from this environment.

That should happen before implementation is considered complete.

### 2. Error UX is different from WezTerm

WezTerm has toast notifications. Kitty does not expose the same UX surface in the current plan.

The most likely v1 answer is `kitten notify` with a terse message.

### 3. Remote detection is weaker than WezTerm

Without domain metadata, the Kitty adapter will likely depend mostly on command-line parsing and explicit host overrides.

That is acceptable for v1, but it should be documented clearly.

### 4. Keybinding defaults must be terminal-specific

PasteHop should not assume all terminals use `CTRL+V` for paste. Kitty should default to Kitty's native paste shortcut.

## Proposed V1 Managed Block

Illustrative only:

```conf
# BEGIN PASTEHOP MANAGED BLOCK
map ctrl+v kitten pastehop.py
map ctrl+shift+v kitten pastehop.py
map cmd+v kitten pastehop.py
# END PASTEHOP MANAGED BLOCK
```

The installer can emit the platform-appropriate subset of these mappings.

## Recommended Acceptance Criteria

Kitty support is ready to merge when:

- `ph install kitty` writes both the kitten and config block
- `ph uninstall kitty` removes both cleanly
- local Kitty sessions preserve native paste
- remote SSH sessions inject uploaded remote paths
- text clipboard sessions do not trigger PasteHop upload logic
- the adapter does not depend on undocumented Kitty internals

## Commands Used In This Spike

Validated against locally installed Kitty `0.46.0`.

Commands used:

```bash
kitty --version
kitten --version
kitten @ --help
kitten @ action -h
kitten @ ls -h
```

I also inspected the official docs bundled with the installed app, including:

- `doc/kitty/html/_sources/kittens/custom.rst.txt`
- `doc/kitty/html/_sources/launch.rst.txt`
- `man/man5/kitty.conf.5`
- `man/man1/kitten-@-action.1`
- `man/man1/kitten-@-ls.1`
- `man/man1/kitten-@-send-text.1`

## Sources

- Kitty custom kittens: https://sw.kovidgoyal.net/kitty/kittens/custom/
- Kitty remote control: https://sw.kovidgoyal.net/kitty/remote-control/
- Kitty actions: https://sw.kovidgoyal.net/kitty/actions/
- Kitty notifications: https://sw.kovidgoyal.net/kitty/kittens/notify/
- Kitty ssh kitten: https://sw.kovidgoyal.net/kitty/kittens/ssh/
