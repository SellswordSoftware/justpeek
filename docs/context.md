# JustPeek Context

## Purpose

JustPeek is a desktop quick-reference utility built with Tauri. It sits in the tray, listens for a global hotkey, detects the active window, and shows a floating panel with context-relevant reference data loaded from local YAML files.

The original concept started as a shortcut viewer, but the current product scope is broader:
- keyboard shortcuts
- CLI command references
- people/name references
- other compact grouped reference material

## Project State

The implementation plan in [shortcutimplementation.md](/home/mike/sellsword/justpeek/docs/shortcutimplementation.md) has been completed, and the app has gone through additional post-plan fixes based on manual testing.

The project is functional now. Most recent work has focused on:
- active-window detection on KDE Wayland
- keeping the panel window preloaded and hidden instead of recreating it each time
- panel drag/resize/transparent styling fixes
- picker fallback improvements
- keyboard-only switch to the manual picker via `Escape`

## Important Current Reality

Some current behavior differs from the original implementation doc and PRD. A new agent should trust the code over the older planning docs where they conflict.

Current reality:
- The panel window is created once at startup and kept hidden.
- The hotkey shows/hides that persistent panel window instead of creating/destroying it each time.
- KDE Wayland detection is implemented using a KWin script plus DBus bridge.
- X11 detection is implemented.
- Windows detection is implemented.
- Sway detection is still stubbed.
- The picker is no longer just a passive fallback list; it is now intended as a manual reference selector.
- The visible `Pick` button was removed. Picker entry is intended to be keyboard-only via `Escape`.

## Product Behavior

Normal flow:
1. App starts tray-first.
2. Backend scans the references directory and starts a watcher.
3. Global hotkey toggles the panel.
4. Backend detects the active window.
5. Backend matches a YAML reference file by process name and optional title regex.
6. Frontend shows grouped reference data with fuzzy filtering.

Manual fallback flow:
1. If detection fails or no file matches, backend sends picker data.
2. Frontend shows the manual picker list.
3. User can select a reference file manually.

Context override flow:
1. If the wrong contextual file is shown, pressing `Escape` is intended to switch from the contextual view to the picker.

## Tech Stack

- Tauri v2
- Rust backend
- Vanilla JS frontend
- `naf` submodule for signals/components
- `nass` submodule for style primitives/foundation

Submodules:
- `src/vendor/naf`
- `src/vendor/nass`

## Frontend Structure

Main entry:
- [src/main.js](/home/mike/sellsword/justpeek/src/main.js)

Key frontend files:
- [src/panel.js](/home/mike/sellsword/justpeek/src/panel.js): floating panel UI, contextual mode, picker mode, fuzzy filtering, keyboard handling
- [src/settings.js](/home/mike/sellsword/justpeek/src/settings.js): settings window UI
- [src/runtime/tauri.js](/home/mike/sellsword/justpeek/src/runtime/tauri.js): wrapper around `window.__TAURI__`
- [src/runtime/naf.js](/home/mike/sellsword/justpeek/src/runtime/naf.js): app-local NAF runtime surface
- [src/runtime/fuzzy.js](/home/mike/sellsword/justpeek/src/runtime/fuzzy.js): fuzzy matcher
- [src/styles/app.css](/home/mike/sellsword/justpeek/src/styles/app.css): panel/settings styling

`main.js` listens for Tauri events:
- `show-panel`
- `show-panel-picker`
- `hide-panel`
- `open-settings`

It mounts either:
- panel UI
- settings UI
- browser fallback scaffold

## Backend Structure

Main backend entry:
- [src-tauri/src/lib.rs](/home/mike/sellsword/justpeek/src-tauri/src/lib.rs)

Important backend modules:
- [src-tauri/src/config.rs](/home/mike/sellsword/justpeek/src-tauri/src/config.rs): config read/write and default paths
- [src-tauri/src/scanner.rs](/home/mike/sellsword/justpeek/src-tauri/src/scanner.rs): YAML parsing, reference lookup, picker app generation
- [src-tauri/src/watcher.rs](/home/mike/sellsword/justpeek/src-tauri/src/watcher.rs): filesystem watch + reload
- [src-tauri/src/hotkey.rs](/home/mike/sellsword/justpeek/src-tauri/src/hotkey.rs): hotkey registration
- [src-tauri/src/detection/mod.rs](/home/mike/sellsword/justpeek/src-tauri/src/detection/mod.rs): detector chain
- [src-tauri/src/detection/windows.rs](/home/mike/sellsword/justpeek/src-tauri/src/detection/windows.rs)
- [src-tauri/src/detection/x11.rs](/home/mike/sellsword/justpeek/src-tauri/src/detection/x11.rs)
- [src-tauri/src/detection/wayland_kde.rs](/home/mike/sellsword/justpeek/src-tauri/src/detection/wayland_kde.rs)
- [src-tauri/src/detection/wayland_sway.rs](/home/mike/sellsword/justpeek/src-tauri/src/detection/wayland_sway.rs)

`AppData` in `lib.rs` currently holds:
- config
- in-memory reference map
- panel window tracking
- panel-ready state
- pending panel payload
- detection chain
- watcher handle

Important Tauri commands:
- `show_panel`
- `hide_panel`
- `get_config`
- `set_config`
- `reload_shortcuts`
- `open_shortcuts_dir`
- `get_picker_apps`
- `load_picker_app`

## Window Detection Notes

### Windows

Uses foreground window APIs and process lookup via the `windows` crate.

### Linux X11

Uses `_NET_ACTIVE_WINDOW`, PID lookup, and title lookup via `x11rb`.

### KDE Wayland

This is important because it was added after the original plan:
- a small KWin script is written to the JustPeek config runtime directory
- the script observes `workspace.activeWindow`
- it sends window updates back to JustPeek over DBus
- Rust stores the last known active window in memory
- detector reads from that cache instead of doing an interactive query

This replaced an earlier broken approach that used KWin `queryWindowInfo()`, which forced the user into a crosshair-and-click flow.

### Sway Wayland

Still a stub returning `None`.

## Matching Logic

Reference files are keyed primarily by process name, with optional title filtering.

Current lookup behavior in [scanner.rs](/home/mike/sellsword/justpeek/src-tauri/src/scanner.rs):
- direct process candidate matching first
- title regex check if present
- deterministic fallback scan across unique files

This code was adjusted because earlier fallback behavior iterated `HashMap` values and felt random.

`WindowInfo` now carries:
- `process_name`
- `window_title`
- `process_candidates`

The candidate list is used to normalize things like:
- `org.wezfurlong.wezterm`
- `wezterm-gui`
- `wezterm`

so existing YAML process names still match on Wayland.

## Reference Data Model

Canonical top-level field:
- `references`

Compatibility alias still supported:
- `shortcuts`

Reference file model:
- `name`
- `process`
- optional `title_pattern`
- `references`
  - `group`
  - `items`
    - optional `keys`
    - `label`
    - optional `value`
    - optional `notes`

Sample files live in:
- [shortcuts-example/vscode.yaml](/home/mike/sellsword/justpeek/shortcuts-example/vscode.yaml)
- [shortcuts-example/git.yaml](/home/mike/sellsword/justpeek/shortcuts-example/git.yaml)
- [shortcuts-example/people.yaml](/home/mike/sellsword/justpeek/shortcuts-example/people.yaml)

## Config and Paths

Config path:
- `~/.config/justpeek/config.yaml`

Default references dir:
- `~/.config/justpeek/references`

Setup helper:
- [setup.sh](/home/mike/sellsword/justpeek/setup.sh)

This copies example reference files into the default references directory.

## UX Notes

Panel behavior:
- transparent window outside the rounded shell
- draggable from header
- resizable via explicit edge/corner resize regions
- header and controls pinned
- reference list scrolls independently

Keyboard behavior:
- arrows move highlight
- `Escape` is intended to switch contextual view to picker
- `Escape` from picker mode closes the panel

The picker flow was still being refined at the end of the last session, so this is one area a new agent should verify directly before making assumptions.

## Known Gaps / Likely Next Work

Known open or likely-fragile areas:
- verify the `Escape`-to-picker path in a live running app after the latest frontend/backend changes
- Sway Wayland detection is still unimplemented
- some README/docs statements still lag behind the current persistent-panel architecture
- hotkey changes in settings still require restart in practice
- active-window handling on Wayland should be re-verified after any KWin or desktop-session changes

## Debugging Notes

The backend emits useful logs to stderr with `[justpeek]`.

Important categories:
- detector selection and active window info
- reference lookup reasoning
- KDE bridge updates
- panel payload selection

When debugging matching problems, start with:
1. what detector ran
2. what process/title/candidates were detected
3. which reference file was selected and why

## How To Verify Safely

Useful checks used during development:

Frontend:
```bash
node --check src/main.js
node --check src/panel.js
node --check src/settings.js
```

Backend:
```bash
CARGO_TARGET_DIR=/tmp/justpeek-cargo-check cargo check --manifest-path src-tauri/Cargo.toml
```

The temporary target dir is useful because a running Tauri app can hold the normal Cargo target lock.

## Suggested Starting Points For A New Agent

If the task is UI/panel behavior:
- start with `src/panel.js`, `src/main.js`, `src/styles/app.css`

If the task is matching/reference logic:
- start with `src-tauri/src/scanner.rs`

If the task is hotkey/show-hide/window behavior:
- start with `src-tauri/src/lib.rs` and `src-tauri/src/hotkey.rs`

If the task is Linux active-window detection:
- start with `src-tauri/src/detection/mod.rs`
- then inspect the platform-specific detector file

If the task is “why did KDE Wayland stop working?”:
- inspect `src-tauri/src/detection/wayland_kde.rs`
- verify the KWin script was written
- verify DBus bridge logs appear

## Bottom Line

JustPeek is already a working tray-based contextual reference app. The main risk areas are no longer core architecture; they are integration details:
- active-window detection edge cases
- picker/manual-selection flow
- platform-specific behavior on Wayland

A new agent should treat the codebase as mature enough to iterate on, but should verify behavior in the running app before assuming any remaining edge case is fully solved.
