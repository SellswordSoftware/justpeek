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

That copies the sample YAML files from `shortcuts-example/` into:

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
- Hotkey changes made in settings currently apply after restarting the app.

## Reference File Format

Example:

```yaml
name: VS Code
process:
  - code
  - Code
references:
  - group: Navigation
    items:
      - keys: Ctrl+P
        label: Quick Open
        notes: Opens a file by name
      - keys: Ctrl+Shift+P
        label: Command Palette
  - group: Git
    items:
      - label: Revert a commit
        value: git revert <commit>
        notes: Safe for shared history
```

Supported fields:

- `name`: display name for the matched reference set
- `process`: optional process name or list of process names for contextual matching
- `title_pattern`: optional regex for window-title matching
- `title_contains`: optional case-insensitive window-title substring matcher
- `references`: list of groups
- `group`: section title
- `items`: list of entries inside the group
- `keys`: optional key chord or list of key chords rendered with `<kbd>`
- `label`: primary text
- `value`: optional secondary value such as a command
- `command`: alias for `value`, recommended for CLI-style entries
- `notes`: optional descriptive text
- `url`: optional supporting link such as docs or a runbook
- `search_terms`: optional list of extra search-only aliases

More detailed docs:

- `docs/referencefile.md`: practical authoring guide for new users
- `docs/reference-schema-proposal.md`: concrete schema improvements and migration direction

## Example Files

The repo includes sample reference packs in:

- `shortcuts-example/vscode.yaml`
- `shortcuts-example/git.yaml`
- `shortcuts-example/people.yaml`

## Current Platform Notes

- Linux X11 active-window detection is implemented.
- Windows active-window detection is implemented.
- Wayland Sway detection is still stubbed.
- Wayland global hotkeys are handled through the desktop portal.

## License

MIT
