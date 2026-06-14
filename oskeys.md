# OS-Specific Shortcut Schema Proposal

## Reader

This note is for the maintainer who wants to add OS-specific shortcut variants without splitting one reference pack into multiple files.

After reading it, the maintainer should be able to decide:

1. whether JustPeek should support OS-specific key variants at the item level
2. what backward-compatible schema shape to implement
3. what settings and rendering rules should accompany that schema

## Problem

Some apps use the same action names across platforms but ship different keybindings:

- macOS: `Cmd+P`
- Windows/Linux: `Ctrl+P`

The current schema models `keys` as one shared list on each item. That works when all platforms use the same binding, but it breaks down when:

- one action has different keys by OS
- a user wants to view only their current platform's bindings
- a cross-platform user wants to see all variants side by side

Duplicating whole items per OS would be noisy and hard to maintain. Duplicating whole files per OS would be worse unless the actual content diverges substantially beyond the bindings.

## Goals

- Keep one logical item per action.
- Allow per-OS key variants on that item.
- Stay backward compatible with existing `keys`.
- Make it possible to show only the current OS, a chosen OS, or all variants.
- Keep manual authoring reasonable in YAML.

## Non-Goals

- Splitting one app reference into separate files per OS by default.
- Modeling arbitrary device-specific input schemes beyond a small set of desktop OS targets.
- Solving every future content-localization problem in the same schema.

## Recommended Schema

Keep `keys` for the current simple case, and add a new optional `keys_by_os` field for platform-specific variants.

### Existing Item

```yaml
- label: Quick Open
  keys:
    - Ctrl+P
```

### Proposed Item

```yaml
- label: Quick Open
  keys_by_os:
    macos:
      - Cmd+P
    windows:
      - Ctrl+P
    linux:
      - Ctrl+P
```

### Proposed Item With Fallback

```yaml
- label: Quick Open
  keys:
    - Ctrl+P
  keys_by_os:
    macos:
      - Cmd+P
```

Meaning:

- `keys` remains the default fallback list
- `keys_by_os` overrides `keys` for the chosen OS when present

## Why `keys_by_os` Instead of Overloading `keys`

This shape is clearer than making `keys` sometimes be a list and sometimes be an object.

Pros:

- easy to understand when reading raw YAML
- easy to validate
- easy to keep backward compatible
- keeps old files unchanged

Cons:

- one more field in the item schema
- slightly more rendering logic

## Supported OS Keys

Use a small explicit set:

- `macos`
- `windows`
- `linux`

Optional future extension:

- `default`

For the first version, `default` is not necessary if `keys` already exists and serves that role.

## Resolution Rules

Given an item and a chosen display OS:

1. If `keys_by_os.<chosen_os>` exists, use that.
2. Else if `keys` exists, use `keys`.
3. Else show no keys for that item.

If the user selects `show all OS variants`:

1. Show each `keys_by_os` entry with an OS label.
2. If `keys` exists and no explicit variant exists for one or more OSes, treat `keys` as the unlabeled fallback or as `default`.

## Recommended Render Behavior

Default behavior should be:

- `Preferred shortcut OS`: `Auto-detect`
- `Shortcut display mode`: `Current OS only`

That means:

- macOS users see macOS bindings if present
- Windows users see Windows bindings if present
- Linux users see Linux bindings if present
- everyone falls back to `keys` when an OS-specific override is not present

## Recommended Settings

Two settings are enough for the first pass.

### Preferred Shortcut OS

Options:

- `Auto-detect`
- `macOS`
- `Windows`
- `Linux`

Purpose:

- lets a cross-platform user force the displayed binding set
- keeps behavior deterministic when users are reading references for a different machine

### Shortcut Display Mode

Options:

- `Current OS only`
- `All OS variants`

Purpose:

- keeps the normal display compact
- gives advanced users a way to inspect every platform binding

## Display Examples

### Current OS Only

```yaml
- label: Quick Open
  keys:
    - Ctrl+P
  keys_by_os:
    macos:
      - Cmd+P
```

Rendered on macOS:

- `Cmd+P`

Rendered on Linux:

- `Ctrl+P`

### All OS Variants

Rendered:

- `macOS: Cmd+P`
- `Default: Ctrl+P`

If all platforms are explicitly defined:

```yaml
- label: Quick Open
  keys_by_os:
    macos:
      - Cmd+P
    windows:
      - Ctrl+P
    linux:
      - Ctrl+P
```

Rendered:

- `macOS: Cmd+P`
- `Windows: Ctrl+P`
- `Linux: Ctrl+P`

## Search Behavior

Recommended rule:

- Search should match both the visible key set and any hidden OS-specific variants.

Reason:

- a user may search for `Cmd+P` while currently viewing Linux bindings
- matching hidden variants makes the data easier to discover

The UI should still render according to the selected display mode, but search indexing should include:

- `keys`
- every value under `keys_by_os`

## TOML Shape

If TOML is ever supported, the same item-level concept can carry over directly:

```toml
[[entries]]
group = "Navigation"
label = "Quick Open"
keys = ["Ctrl+P"]

[entries.keys_by_os]
macos = ["Cmd+P"]
```

This is workable, though still more verbose than YAML.

## Backward Compatibility

Existing files should continue to work unchanged.

Compatibility rules:

- `keys` alone remains valid
- `keys_by_os` is optional
- if both exist, `keys_by_os` only overrides the selected OS

No migration should be required for current user files.

## Validation Rules

Recommended validation:

- `keys` remains a string or list of strings, as it is today
- `keys_by_os` must be an object
- each OS key under `keys_by_os` must map to a string or list of strings
- unknown OS names should be rejected or ignored with a warning

## Why Not Separate Files Per OS

Separate files such as `vscode-macos.yaml` and `vscode-linux.yaml` are only worth it when the content itself differs materially.

They are a poor fit when only the keybinding changes because they:

- duplicate labels, notes, and URLs
- make updates harder to keep in sync
- clutter the picker
- complicate process matching

## Recommendation

The best next step is:

1. keep the existing YAML format
2. add optional `keys_by_os` per item
3. keep `keys` as the fallback
4. add two settings:
   - `Preferred shortcut OS`
   - `Shortcut display mode`

This gives JustPeek:

- backward compatibility
- compact authoring
- per-platform flexibility
- a simple display model

## Implementation Plan

This plan assumes the goal is to ship OS-specific shortcut support incrementally without breaking existing reference files.

### Phase 1: Extend the Data Model

Add a new optional `keys_by_os` field to each reference item.

Target shape:

```yaml
- label: Quick Open
  keys:
    - Ctrl+P
  keys_by_os:
    macos:
      - Cmd+P
```

Required behavior:

- `keys` continues to work exactly as it does today
- `keys_by_os` is optional
- existing files require no migration

Implementation tasks:

1. Extend the reference item model to include `keys_by_os`.
2. Reuse the current string-or-list parsing rules for each OS entry.
3. Accept only known OS names:
   - `macos`
   - `windows`
   - `linux`
4. Decide whether unknown OS keys should be rejected or ignored with a warning.

Recommended decision:

- reject invalid OS names during parsing if the file is otherwise valid enough to report an actionable error

### Phase 2: Add Runtime Resolution Rules

Introduce one resolver that converts raw item data into the key set that should be displayed for the current session.

Resolver inputs:

- the item
- the effective preferred OS
- the display mode

Resolver outputs:

- the key list to display
- optional OS labels when showing all variants

Resolution rules for `Current OS only`:

1. use `keys_by_os.<preferred_os>` if present
2. else use `keys`
3. else show no keys

Resolution rules for `All OS variants`:

1. collect every entry under `keys_by_os`
2. if `keys` exists, include it as `Default` or unlabeled fallback
3. preserve a deterministic order:
   - `macOS`
   - `Windows`
   - `Linux`
   - `Default`

Implementation tasks:

1. Add a resolver helper for key display.
2. Keep the raw parsed data separate from the rendered display form.
3. Ensure the same resolver is used everywhere shortcut keys are shown.

### Phase 3: Add Settings

Add two user-facing settings.

#### Preferred Shortcut OS

Options:

- `Auto-detect`
- `macOS`
- `Windows`
- `Linux`

Behavior:

- `Auto-detect` maps from the current runtime OS
- explicit choices override runtime detection

#### Shortcut Display Mode

Options:

- `Current OS only`
- `All OS variants`

Implementation tasks:

1. Extend the config model with two new fields.
2. Add defaults for first run:
   - preferred OS: `auto`
   - display mode: `current`
3. Add controls to the settings window.
4. Save and reload those settings through the existing config flow.

### Phase 4: Update Rendering

The panel should render the resolved key list rather than the raw `keys` field.

Required behaviors:

- current mode remains compact and familiar
- all-variants mode displays OS labels clearly
- items with no keys still render normally

Recommended rendering for `All OS variants`:

- `macOS: Cmd+P`
- `Windows: Ctrl+P`
- `Linux: Ctrl+P`

Implementation tasks:

1. Update item rendering to consume resolved key display data.
2. Add a small OS label style for all-variants mode.
3. Keep the current visual treatment for normal single-platform display.

### Phase 5: Update Search Indexing

Search should match both visible and hidden OS-specific variants.

Required behavior:

- a Linux user searching for `Cmd+P` should still find the relevant action if the item has a macOS variant

Implementation tasks:

1. Include `keys`
2. Include every value under `keys_by_os`
3. Keep rendering filtered by display mode even if search matched a hidden variant

### Phase 6: Validation and Error Handling

Add clear behavior for malformed files.

Validation targets:

- `keys_by_os` must be a mapping
- each OS entry must be a string or string list
- empty strings should be dropped the same way current key parsing drops them

Recommended error strategy:

- invalid item-level shortcut data should reject the file with a clear parse error
- avoid silently half-parsing ambiguous structures

### Phase 7: Documentation and Examples

Update user-facing docs once the feature is implemented.

Docs to update:

1. reference file format examples
2. settings documentation
3. at least one sample reference file using `keys_by_os`

Recommended sample:

- VS Code or another app with clear macOS vs Windows/Linux bindings

### Phase 8: Verification

Minimum verification should cover parsing, resolution, rendering, and settings persistence.

#### Parser Tests

- file with only `keys`
- file with only `keys_by_os`
- file with both `keys` and `keys_by_os`
- file with invalid OS names
- file with invalid `keys_by_os` value types

#### Resolver Tests

- current OS match
- fallback to `keys`
- all-variants ordering
- item with no matching OS variant and no fallback

#### UI Tests

- current OS only renders one key set
- all-variants mode renders multiple labeled sets
- search matches hidden OS-specific variants
- settings changes persist and affect display immediately or on next panel open, depending on the chosen settings model

## Suggested Delivery Order

Ship in this order:

1. parser and internal data model
2. runtime key resolver
3. panel rendering for current OS only
4. settings for preferred OS
5. all-variants display mode
6. search indexing updates
7. docs and sample files

Reason:

- the current-OS-only path delivers most of the value first
- all-variants mode is useful, but not necessary for the initial feature
- shipping parser plus resolver early keeps the schema stable before UI polish work

## Lowest-Risk First Release

If this should land in a smaller first release, ship only:

1. `keys_by_os`
2. preferred OS setting
3. current OS only display
4. search across all variants

Defer:

- all-variants display mode
- OS label styling
- more advanced config behavior

That release already solves the main user problem while keeping the UI simple.

## Task List

This task list maps the plan onto the current JustPeek codebase.

### Slice A: Extend the Rust Reference Schema

- [ ] Add `keys_by_os` to the Rust `ReferenceItem` model in `scanner.rs`.
- [ ] Add a parser for OS-keyed shortcut variants that reuses the current string-or-list handling for `keys`.
- [ ] Restrict supported OS keys to `macos`, `windows`, and `linux`.
- [ ] Decide parser behavior for unknown OS keys and implement it consistently.
- [ ] Add unit tests for:
  - `keys` only
  - `keys_by_os` only
  - both `keys` and `keys_by_os`
  - invalid OS names
  - invalid `keys_by_os` value shapes

After this:

- JustPeek can parse OS-specific shortcut variants without breaking existing files.

### Slice B: Extend the Frontend Item Shape

- [ ] Update the frontend `PanelItem` typedef in `src/panel.js` to represent OS-specific key data.
- [ ] Decide whether frontend payloads should carry:
  - raw `keys` plus raw `keys_by_os`, or
  - already-resolved display keys from Rust
- [ ] Keep the raw-vs-resolved contract explicit so rendering and search do not drift.

Recommended decision:

- keep raw data in Rust and resolve display behavior in the frontend only if the setting is purely presentation-driven
- otherwise resolve in Rust if you want one canonical display contract from backend to UI

After this:

- The JS side can represent OS-specific key data without forcing an immediate UI change.

### Slice C: Add Config Fields for OS Preference

- [ ] Extend `Config` in `src-tauri/src/config.rs` with:
  - preferred shortcut OS
  - shortcut display mode
- [ ] Add defaults for both fields in `Config::default()`.
- [ ] Ensure config read/write remains backward compatible with older `config.yaml` files.
- [ ] Extend the config payload shape exposed through `src/runtime/tauri.js`.
- [ ] Add or update tests for the config contract if present.

Suggested values:

- preferred shortcut OS: `auto`, `macos`, `windows`, `linux`
- display mode: `current`, `all`

After this:

- The app can persist the settings needed to drive OS-specific shortcut display.

### Slice D: Surface the Settings in the Settings Window

- [ ] Add frontend state in `src/settings.js` for:
  - preferred shortcut OS
  - shortcut display mode
- [ ] Load those values from `getConfig()`.
- [ ] Save them through `setConfig()`.
- [ ] Add controls to the settings form:
  - select for preferred shortcut OS
  - select or toggle for shortcut display mode
- [ ] Update `src/styles/app.css` if the settings layout needs additional spacing or wrapping.

After this:

- A user can control shortcut OS resolution from the settings UI.

### Slice E: Implement Current-OS Key Resolution

- [ ] Add one resolver for current-OS display.
- [ ] Resolver rule:
  - use `keys_by_os.<preferred_os>` when present
  - otherwise fall back to `keys`
- [ ] Determine how `auto` maps to runtime OS.
- [ ] Decide where runtime OS should come from:
  - reuse existing backend runtime info, or
  - expose a frontend runtime OS field if needed

Likely touch points:

- `src-tauri/src/lib.rs` runtime info surface
- `src/runtime/tauri.js`
- `src/panel.js`

After this:

- The panel can show the correct keys for one effective OS without showing all variants.

### Slice F: Update Panel Rendering for Resolved Keys

- [ ] Replace direct rendering of `item.keys` in `src/panel.js` with resolved display keys.
- [ ] Preserve the current look when only one key set is displayed.
- [ ] Ensure items with no displayable keys still render cleanly.
- [ ] Verify that keyboard navigation, copy behavior, and current row highlighting remain unchanged.

After this:

- OS-specific bindings appear in the panel for the selected platform without changing the surrounding UX.

### Slice G: Update Search Indexing

- [ ] Extend the contextual fuzzy-search key source in `src/panel.js`.
- [ ] Include:
  - fallback `keys`
  - all values from `keys_by_os`
- [ ] Keep rendering filtered by the chosen display mode even if search matched a hidden OS variant.

After this:

- Searching for a hidden platform-specific binding still finds the right action.

### Slice H: Add All-Variants Display Mode

- [ ] Extend the resolver to return multiple labeled key sets when display mode is `all`.
- [ ] Decide the output order:
  - `macOS`
  - `Windows`
  - `Linux`
  - `Default`
- [ ] Update `renderKeys()` or add a new render path for labeled OS groups.
- [ ] Add light styling for OS labels in `src/styles/app.css`.

After this:

- The panel can show every OS-specific binding on one item in a readable way.

### Slice I: Add Samples and Documentation

- [ ] Update the reference file docs with a `keys_by_os` example.
- [ ] Add at least one sample reference file that uses OS-specific keys.
- [ ] Update `README.md` if the feature should be discoverable there.
- [ ] Update this proposal doc if implementation details diverge from the recommendation.

After this:

- Maintainers and users can discover and author the feature correctly.

### Slice J: Verification Pass

- [ ] Run Rust parser tests for the new schema.
- [ ] Run frontend typecheck after panel and settings updates.
- [ ] Manually verify:
  - current OS only mode
  - explicit preferred OS override
  - all variants mode
  - fallback from `keys_by_os` to `keys`
  - search hits hidden variants
  - old YAML files still render unchanged

After this:

- The feature is verified end to end across schema, settings, rendering, and search.

## Suggested Execution Order

Use this dependency order:

1. Slice A
2. Slice B
3. Slice C
4. Slice D
5. Slice E
6. Slice F
7. Slice G
8. Slice H
9. Slice I
10. Slice J

## Smallest Useful First Milestone

If you want the thinnest useful first implementation, stop after:

- Slice A
- Slice C
- Slice D
- Slice E
- Slice F
- Slice G

That yields:

- `keys_by_os` support
- preferred OS selection
- current-OS-only rendering
- search across all key variants

and leaves all-variants display for a second pass.

## Example Full File

```yaml
name: VS Code
process:
  - code
  - Code

references:
  - group: Navigation
    items:
      - label: Quick Open
        keys:
          - Ctrl+P
        keys_by_os:
          macos:
            - Cmd+P
        notes: Opens a file by name

      - label: Command Palette
        keys_by_os:
          macos:
            - Cmd+Shift+P
          windows:
            - Ctrl+Shift+P
          linux:
            - Ctrl+Shift+P

  - group: Terminal
    items:
      - label: Toggle Terminal
        keys_by_os:
          macos:
            - Ctrl+`
          windows:
            - Ctrl+`
          linux:
            - Ctrl+`
```
