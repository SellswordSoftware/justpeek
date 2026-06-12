# Shortcut Viewer — Product Requirements Document

## Problem Statement

Power users often need quick-reference information that is relevant to the application or task in front of them, but not frequently enough to keep every detail in memory. Keyboard shortcuts are the clearest example, but the same pattern applies to CLI command syntax, app-specific workflows, naming/reference lists for recurring communications, and other contextual cheat-sheet data. That information is usually scattered across docs, notes, cheat sheets, or muscle memory fragments, and there is no consistent, universal way to instantly surface it for the currently active application without context-switching away from the workflow.

## Solution

A lightweight, cross-platform desktop utility that runs in the background and listens for a configurable global hotkey. When triggered, it detects the currently active application, looks up a matching quick-reference file from a local configuration directory, and presents the data in a small, always-on-top floating panel with a fuzzy search filter. The panel is designed to be a glanceable reference surface: sometimes that means keyboard shortcuts, but it can also mean command syntax, naming aids, or other structured reference notes tied to the current app or context.

## User Stories

1. As a power user, I want to press a global hotkey to instantly see reference information for my currently active application, so that I can get the detail I need without leaving my workflow.
2. As a user, I want the reference panel to appear as a borderless, always-on-top floating window, so that it can sit on top of any application without interfering with my work.
3. As a user, I want to be able to drag and reposition the panel anywhere on screen, so that I can place it in an unobtrusive location while I work.
4. As a user, I want to press the same global hotkey again to dismiss the panel, so that toggling it open and closed is fast and symmetrical.
5. As a user, I want a close button on the panel, so that I can dismiss it with my mouse if I prefer.
6. As a user, I want the panel to remain open and visible even when I click into another application, so that I can reference the data while working.
7. As a user, I want to type into a fuzzy search input to filter entries by label, key combination, value, or notes, so that I can quickly find specific reference items among dozens of options.
8. As a user, I want reference groups to be collapsible by clicking the group header, so that I can hide irrelevant sections.
9. As a user, I want to navigate filtered entries with arrow keys and see a highlighted row, so that I can use the panel without a mouse.
10. As a user, I want keyboard combinations rendered with distinct `<kbd>` styling when entry data includes a dedicated `keys` field, so that shortcut-oriented content remains visually distinct and easy to read.
11. As a user, I want to write reference files in a human-readable YAML format, so that I can easily create and edit reference definitions for any application.
12. As a user, I want reference files to be organized into named groups within each file, so that entries are logically categorized (e.g., Navigation, Git Recovery, People).
13. As a user, I want to specify the process name(s) in each reference file, so that the app knows which file to load for a given application.
14. As a user, I want to optionally specify a window title regex pattern in reference files, so that I can differentiate references for different windows of the same application.
15. As a user, I want the app to scan a single config directory recursively for all reference files at startup, so that I can organize them into subdirectories as I see fit.
16. As a user, I want the app to automatically reload reference files when they are created, modified, or deleted, so that my changes take effect immediately without restarting the app.
17. As a user, I want a tray icon with options to quit, reload reference files, and open the config directory, so that I can manage the app lifecycle from the system tray.
18. As a user, I want to configure the global hotkey via a simple config file, so that I can choose a key combination that does not conflict with my other applications.
19. As a user, I want to configure the panel theme (dark/light) via the config file, so that the panel matches my preference or environment.
20. As a user, I want a settings window accessible from the tray menu to edit configuration values, so that I can manage settings without manually editing files.
21. As a user on Windows, I want the app to automatically detect the foreground window and show its matching reference data, so that the experience is seamless on Windows.
22. As a user on Linux (X11), I want the app to automatically detect the foreground window and show its matching reference data, so that the experience is seamless on X11-based desktops.
23. As a user on Linux (Wayland), I want the app to attempt compositor-specific detection (Sway, KDE) before falling back to a quick app picker, so that I can still use the app on Wayland.
24. As a user, I want to see a quick picker of all known reference files when no active window match is found, so that I can still access reference data manually.
25. As a user, I want the "no match found" message to display the detected process name and window title, so that I know exactly what the app detected and can create a matching reference file.
26. As a developer, I want the frontend to use a vanilla JS reactive system (NAF) with no external UI dependencies, so that the app remains lightweight and maintainable.
27. As a developer, I want the panel window to be created fresh on each hotkey press and destroyed on close, so that no memory is wasted when the panel is not in use.
28. As a developer, I want the Rust backend to hold the reference data map and pass matched data to the frontend on demand, so that there is a single source of truth and the frontend stays lightweight.
29. As a user, I want the panel to have a transparent background with rounded corners and a frosted glass blur effect, so that it looks modern and blends naturally with the underlying application.

## Implementation Decisions

### Core Architecture
- **Standalone Tauri v2 application** with Rust backend and vanilla JS frontend.
- **Inline Rust commands** in `src-tauri/src/lib.rs` (not a separate plugin). Commands: `get_config`, `set_config`, `open_references_dir`, `reload_references`, `hide_panel`.
- **Frontend bootstrap pattern:** `createPanel(data)` factory function returns a self-contained NAF component with `mount()` and `unmount()` lifecycle. The JS entry point listens for `show-panel` and `hide-panel` Tauri events.

### Data Flow
- Rust maintains an in-memory `HashMap<String, ShortcutFile>` keyed by process name(s).
- Reference files are YAML-parsed at startup and on filesystem watch events.
- On hotkey press: Rust detects active window → looks up matching reference data → creates Tauri window → emits `show-panel` event with matched data.
- Frontend receives data, populates NAF signals, and renders the panel component tree.

### Window Detection
- **Windows:** `GetForegroundWindow` via `windows` crate. Retrieves process name and window title.
- **Linux X11:** `_NET_ACTIVE_WINDOW` via `x11rb` crate. Retrieves PID (for process name) and window name.
- **Linux Wayland:** Attempts Sway IPC (Unix socket) and KDE Portal (D-Bus). Falls back to universal picker if unavailable.
- All platforms fall back to showing the app picker UI if detection fails or no match is found.

### Reference File Format (YAML)
```yaml
name: VS Code
process:
  - code
  - Code
title_pattern: (?-i).*workspace.*
references:
  - group: Navigation
    items:
      - keys: Ctrl+P
        label: Quick Open
        notes: Opens a file by name
  - group: Git
    items:
      - label: Tag current commit
        value: git tag -a v1.2.3 -m "Release v1.2.3"
        notes: Annotated tag
  - group: Team
    items:
      - label: Jordan Lee
        value: Platform lead
        notes: Owns release sign-off and customer escalation triage
```

- `references` is the top-level collection shown in the panel.
- Each item supports flexible fields so the format works for shortcuts, commands, people, and other quick-reference data.
- `keys` is optional and reserved for keyboard combinations. When present, the renderer should split it into parts and display each part using `<kbd>`.
- `label` is the primary human-readable title for an item.
- `value` is optional structured content such as a command, alias, email address, or snippet.
- `notes` is optional supporting context shown in secondary text.
- For backwards compatibility, `shortcuts` may be accepted as an alias for `references` during early development if that reduces migration friction.

### File Discovery & Watching
- Default directory: `~/.config/shortcut-viewer/references/` (Linux), `%APPDATA%/shortcut-viewer/references/` (Windows).
- Recursive scan for `*.yaml`/`*.yml` files at startup.
- `notify` crate watches the directory for changes. Debounced 300ms to avoid partial-write race conditions.

### Panel Window Configuration
- `decorations: false`, `transparent: true`, `always_on_top: true`.
- CSS styling: `border-radius: 12px`, `box-shadow`, `backdrop-filter: blur(8px)`, semi-transparent dark background.
- Draggable header strip via `-webkit-app-region: drag`. Close button and content area are `no-drag`.
- Single-window guarantee: Rust tracks the panel `WindowId`. Hotkey handler toggles: destroys if exists, creates if not.

### Configuration
- `config.yaml` at `~/.config/shortcut-viewer/config.yaml` (Linux) / `%APPDATA%/shortcut-viewer/config.yaml` (Windows).
- Fields: `hotkey` (string, Tauri accelerator syntax), `theme` (string: "dark"|"light"|"system"), `references_dir` (optional path override).
- Default hotkey: `CommandOrControl+Alt+Slash`.

### Fuzzy Search
- Custom built-in fuzzy matching function (~30 lines). Matches query characters in order within target string. Scores based on contiguous runs. Filters against `keys`, `label`, `value`, and `notes`.

### Settings Window
- Separate Tauri window opened from tray menu.
- Fields: hotkey input, theme selector, reference directory path with browse button.
- Saves via `set_config` Rust command, which re-registers the hotkey and restarts the watcher immediately.

## Testing Decisions

### What Makes a Good Test
- Tests verify external behavior (API contracts, data flow), not internal implementation details.
- Fuzzy matcher tested with exhaustive edge cases: empty strings, partial matches, case sensitivity, special characters, and mixed field types.
- Window detection mocked for cross-platform consistency.
- File scanner tested with temporary directory fixtures containing valid, invalid, and edge-case YAML files.

### Modules to Test
1. **Fuzzy search function:** Input/output pairs for match/non-match/scoring.
2. **YAML parser:** Valid files, malformed YAML, missing required fields, empty files.
3. **File scanner:** Recursive directory traversal, extension filtering, debounced watch events.
4. **Reference lookup:** Process name matching (case-insensitive), title pattern regex matching, no-match fallback.
5. **Config management:** Read/write config.yaml, default values, validation.

### Prior Art
- No existing test suite in this project (new application). Tests should be written using standard Rust `cargo test` for backend modules and a lightweight JS test runner (e.g., `node --test`) for the frontend fuzzy matcher.

## Out of Scope

- Inline editing of reference files from within the panel. Editing is read-only; users must open files externally.
- macOS support in v1. Windows and Linux only.
- System-wide reference file installation or marketplace.
- Cloud sync or multi-machine configuration sharing.
- Native window management on Wayland beyond the fallback picker.
- Keyboard shortcut execution or command execution (the app only displays reference material; it does not simulate keypresses or run commands).

## Further Notes

- The app is designed to be a thin, fast utility. Every architectural decision prioritizes low memory overhead and zero unnecessary dependencies.
- The NAF reactive system is reused from the author's existing codebase, providing a proven signal/computed/effect pattern without framework overhead.
- The transparent frosted-glass panel aesthetic is contingent on OS/browser support for `backdrop-filter`. A solid dark background should be used as a fallback where blur is unsupported.
- Future enhancements (out of scope for v1): inline editing, macOS support, reference file sharing/export, per-profile reference sets.
