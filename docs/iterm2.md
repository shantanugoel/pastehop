# iTerm2 Integration Plan

Date: March 12, 2026
Status: Implemented in repo; live iTerm2 validation still pending

## Summary

iTerm2 is a viable direct-integration target for PasteHop on macOS.

The smoothest v1 should update a real iTerm2 profile in place instead of
creating a separate dynamic profile. Dynamic profiles are useful for discovery
and experimentation, but they still behave as separate profiles and would push
the user toward manual profile switching or changing defaults.

The main UX goal for `ph install iterm2` is:

- no manual editing in iTerm2 settings
- no profile-name guesswork in the common case
- a single install command that either succeeds or presents a clear chooser

## Implementation Notes

- The installer updates the real `com.googlecode.iterm2.plist` profile in place.
- The runtime bridge uses an AutoLaunch Python RPC and iTerm2's official scripting API.
- `ph install iterm2` now fails early if iTerm2's Python runtime is missing instead of leaving a dead keybinding behind.
- A restart is still required if iTerm2 was already running when the bridge script was installed.

## Profile Resolution

Target profile resolution should be intentionally simple:

1. If `--profile <name-or-guid>` is provided, use that exact match.
2. Otherwise, if exactly one iTerm2 profile exists, use it automatically.
3. Otherwise, present an interactive chooser and ask the user which profile to update.

That is the whole rule.

The installer should not implicitly choose:

- the current profile
- the default profile
- the most recently used profile

If multiple profiles exist and the command is non-interactive, the install
should fail with a clear message telling the user to rerun with `--profile`.

## Recommended Adapter Design

### Install target

Install against the user's existing iTerm2 preferences plist:

- `~/Library/Preferences/com.googlecode.iterm2.plist`

Relevant data model:

- profiles live under `New Bookmarks`
- each profile has a `Name`
- each profile has a `Guid`
- each profile has a `Keyboard Map`

PasteHop should modify only the selected profile's `Keyboard Map` and keep the
rest of the profile untouched.

### Managed artifacts

The iTerm2 installer should manage these artifacts:

1. a PasteHop bridge helper under an iTerm2-appropriate support directory
2. a manifest recording which profile GUID was updated
3. PasteHop-owned key mapping entries inside the selected profile

The manifest is important so `ph uninstall iterm2` can remove the integration
without asking the user to choose again in the common case.

### Runtime behavior

Add a new hook target:

- `ph hook iterm2`

The iTerm2 bridge should:

1. gather current-session metadata exposed by iTerm2
2. pass normalized metadata into `ph hook iterm2`
3. map hook responses back to iTerm2 behavior

Expected response mapping:

- `inject_text`: paste the returned remote path into the originating session
- `passthrough_key`: perform iTerm2's normal paste behavior
- `noop`: return silently
- `error`: surface a terse local error

The hook path should reuse existing host-detection logic by passing the
foreground SSH command line into the existing parser when possible.

## Implementation Plan

### Phase 1: CLI and module plumbing

Files:

- `src/cli.rs`
- `src/app.rs`
- `src/install.rs`
- `src/terminal/mod.rs`
- `src/terminal/iterm2.rs`

Tasks:

- [x] add `SupportedTerminal::Iterm2`
- [x] add `HookTerminal::Iterm2`
- [x] add `--profile <name-or-guid>` to `ph install iterm2`
- [x] add `--list-profiles` as a discovery aid
- [x] add macOS-only iTerm2 path helpers and profile enumeration

### Phase 2: Profile discovery and chooser

Files:

- `src/terminal/iterm2.rs`
- `src/install.rs`

Tasks:

- [x] parse the iTerm2 plist and enumerate profiles by name and GUID
- [x] support exact matching by name or GUID for `--profile`
- [x] auto-select when exactly one profile exists
- [x] present a numbered interactive chooser when multiple profiles exist
- [x] fail cleanly when multiple profiles exist but stdin is not interactive

Chooser requirements:

- [x] show profile name and GUID
- [x] preserve stable ordering
- [x] make cancellation a safe no-op
- [x] never update more than one profile in a single install

### Phase 3: Installer and uninstaller

Files:

- `src/install.rs`
- `src/terminal/iterm2.rs`
- `assets/iterm2/...`

Tasks:

- [x] back up the original plist once, similar to existing installers
- [x] add only PasteHop-managed key mappings to the chosen profile
- [x] write a manifest with the chosen profile GUID and managed mapping IDs
- [x] make install idempotent
- [x] make uninstall remove only PasteHop-managed mappings
- [x] if the manifest is missing, fall back to scanning for managed mappings

### Phase 4: Runtime bridge

Files:

- `assets/iterm2/...`
- `src/app.rs`
- `src/target.rs`

Tasks:

- [x] implement the bridge helper that invokes `ph hook iterm2`
- [x] collect the foreground command line and any useful session metadata iTerm2 exposes
- [x] reuse the existing SSH parsing path where possible
- [x] preserve normal paste behavior on passthrough

This phase should begin with a small spike to capture one known-good iTerm2 key
mapping payload and verify the bridge can both read session metadata and replay
native paste cleanly.

- [x] capture and use a known-good key mapping payload from a real iTerm2 plist
- [ ] live-window manual validation of native paste replay is still pending

### Phase 5: Tests and docs

Files:

- `src/install.rs`
- `src/cli.rs`
- `README.md`
- `docs/PLAN.md`
- `docs/PRODUCT_SPEC.md`

Tasks:

- [x] add fixture-based tests for multi-profile plist parsing
- [x] test `--profile` matching by name and GUID
- [x] test single-profile auto-selection
- [x] test chooser-only behavior when multiple profiles exist
- [x] test uninstall via saved manifest
- [x] document the macOS-only scope and any one-time limitations

## Recommended Acceptance Criteria

iTerm2 support is ready to merge when:

- `ph install iterm2 --profile <name-or-guid>` updates exactly that profile
- `ph install iterm2` auto-selects only when exactly one profile exists
- `ph install iterm2` presents a chooser when multiple profiles exist
- non-interactive installs with multiple profiles fail with a clear `--profile` instruction
- `ph uninstall iterm2` removes only PasteHop-managed changes
- local iTerm2 sessions preserve normal paste behavior
- remote SSH sessions inject uploaded remote paths

## Sources

- iTerm2 dynamic profiles: https://iterm2.com/documentation-dynamic-profiles.html
- iTerm2 scripting fundamentals: https://iterm2.com/documentation-scripting-fundamentals.html
- iTerm2 variables: https://iterm2.com/documentation-variables.html
- iTerm2 coprocesses: https://iterm2.com/documentation-coprocesses.html
- iTerm2 scripting overview: https://iterm2.com/documentation-scripting.html
