# JustPeek

Contextual quick-reference utility for desktop apps.

JustPeek runs in the system tray, listens for a global hotkey, detects the active window, and shows a floating panel with relevant reference data. Keyboard shortcuts are a first-class use case, but the same YAML format also works for command syntax, people references, notes, and other compact lookup data.

## Features

- Tray-first desktop app built with Tauri
- Global hotkey to toggle the reference panel
- Active-window matching by process name and optional title pattern
- YAML-based grouped reference files
- Fuzzy filtering across `keys`, `label`, `value`, and `notes`
- Dedicated floating panel window with drag and resize support
- Filesystem watching for live reference reloads
- Settings window for hotkey, theme, and references directory

## Run

Install dependencies and start the app:

```bash
npm install
npm run dev
```

Build a desktop bundle:

```bash
npm run build
```

## First-Time Setup

Seed the default references directory with example files:

```bash
./setup.sh
```

That copies the sample YAML files from `examples/` into:

```text
~/.config/justpeek/references
```

If `XDG_CONFIG_HOME` is set, JustPeek uses:

```text
$XDG_CONFIG_HOME/justpeek/references
```

## Configuration

JustPeek stores config at:

```text
~/.config/justpeek/config.yaml
```

Example:

```yaml
hotkey: CommandOrControl+Alt+Slash
theme: dark
references_dir: null
```

Notes:

- `references_dir: null` uses the default JustPeek references directory.
- On Wayland, the desktop portal may prompt you to approve the global shortcut the first time the app starts.
- On Linux Wayland, the hotkey field is hidden in settings because the shortcut is managed by the desktop environment instead of directly by the app.
- Hotkey changes made in settings currently apply after restarting the app.

## Hotkey Format

When the global shortcut is managed by JustPeek itself, enter the hotkey using Tauri-style accelerator syntax:

- Join parts with `+`, for example `CommandOrControl+Alt+Slash`
- Put modifiers first, then the final key
- Use one final key per shortcut

Supported modifier names used by the app:

- `CommandOrControl`
- `Control` or `Ctrl`
- `Alt` or `Option`
- `Shift`
- `Meta`, `Super`, `Command`, or `Cmd`

Common final key names supported by the current hotkey parser:

- letters and digits such as `K`, `P`, or `7`
- `Slash`
- `Question`
- `Space`
- `Enter` or `Return`
- `Escape` or `Esc`
- `Tab`
- `Up`, `Down`, `Left`, `Right`

Examples:

- `CommandOrControl+Alt+Slash`
- `Ctrl+Shift+K`
- `Super+Space`
- `Alt+Enter`

For the most predictable behavior across environments, prefer simple modifier combinations with a single letter, digit, arrow key, or one of the named keys above.

## Reference File Format

Example:

```yaml
name: VS Code Workspace
group: Editors
process:
  - code
  - Code
title_pattern: ^.+ - Visual Studio Code$
title_contains: project
references:
  - group: Navigation
    items:
      - label: Quick Open
        keys: Ctrl+P
        keys_by_os:
          macos: Cmd+P
          windows: Ctrl+P
          linux: Ctrl+P
        notes: Opens a file by name
        search_terms:
          - files
          - open file
      - label: Command Palette
        keys_by_os:
          macos: Cmd+Shift+P
          windows: Ctrl+Shift+P
          linux: Ctrl+Shift+P
        notes: Opens the action menu
  - group: Cli
    items:
      - label: Open VS Code in Current Directory
        value: code .
        notes: Easy way to launch vs code from the terminal
        url: https://code.visualstudio.com/docs/configure/command-line
      - label: VS Code Help
        command: code -h
        notes: cli help output
```

Multi-line notes are supported with YAML block syntax and render with visible line breaks in the panel:

```yaml
- label: Incident handoff
  notes: |
    Check the dashboard first.
    Then review the most recent deploy.
    Escalate if error rate is still rising.
```

Multi-line commands are supported the same way and render with visible line breaks in the command block:

```yaml
- label: Release steps
  command: |
    git tag -a v1.2.3 -m "Release v1.2.3"
    git push --tags
```

`keys_by_os` is optional. If JustPeek has a variant for the preferred or current OS, it uses that. Otherwise it falls back to `keys`.

Supported fields:

- `name`: display name for the matched reference set
- `group`: optional picker category used to group reference files in the picker
- `process`: optional process name or list of process names for contextual matching
- `title_pattern`: optional regex for window-title matching
- `title_contains`: optional case-insensitive window-title substring matcher
- `references`: list of groups
- `group`: section title
- `items`: list of entries inside the group
- `keys`: optional key chord or list of key chords rendered with `<kbd>`
- `keys_by_os`: optional OS-specific key overrides for `macos`, `windows`, and `linux`
- `label`: primary text
- `value`: optional secondary text
- `command`: optional command text rendered in a command block
- `notes`: optional descriptive text
- `url`: optional supporting link such as docs or a runbook
- `search_terms`: optional list of extra search-only aliases

More detailed docs:

- `docs/referencefile.md`: practical authoring guide for new users
- `docs/reference-schema-proposal.md`: concrete schema improvements and migration direction
- `oskeys.md`: proposal for OS-specific shortcut variants and display settings

## Example Files

The repo includes sample reference packs in:

- `examples/vscode.yaml`
- `examples/git.yaml`
- `examples/people.yaml`

## Current Platform Notes

- Linux X11 active-window detection is implemented.
- Windows active-window detection is implemented.
- Wayland Sway detection is still stubbed.
- Wayland global hotkeys are handled through the desktop portal.

## Window Placement

On KDE Plasma, restored panel placement may be controlled by the compositor rather than by JustPeek.

If the panel reopens centered instead of where you last left it, check:

`System Settings > Window Management > Window Behavior > Advanced > Windows Placement`

If you want KDE Plasma to restore the window position it had when closed, you may also want an additional compositor helper such as:

- [RememberWindowPositions](https://github.com/rxappdev/RememberWindowPositions)

## License

MIT
