# JustPeek Reference Files

## Audience

This guide is for someone creating or editing a JustPeek reference file for the first time.

## Goal

After reading this, you should be able to create a reference file that:

- shows up in JustPeek
- matches the right app when appropriate
- stays easy to search and maintain

## What a Reference File Does

A reference file is a small YAML document that tells JustPeek:

- what the reference set is called
- which app or window it should match, if any
- how to group the content
- what each entry should show in the panel

Reference files are useful for more than keyboard shortcuts. They also work well for:

- command references
- team directories
- incident notes
- compact lookup data

## Smallest Useful Example

```yaml
name: VS Code
group: Editors
process:
  - code
references:
  - group: Navigation
    items:
      - label: Quick Open
        keys: Ctrl+P
        notes: Opens a file by name
```

This file says:

- the panel title should be `VS Code`
- it should match the `code` process
- it has one group called `Navigation`
- that group has one entry

## Full Current Schema

The current schema supports these fields.

### Top level

- `name`: required display name for the reference set
- `group`: optional picker category used to group files in the manual reference picker
- `process`: optional process name or list of process names used for contextual matching
- `title_pattern`: optional regex for narrowing a `process` match by window title
- `title_contains`: optional simpler substring matcher for narrowing a `process` match by window title
- `references`: required list of reference groups
- `shortcuts`: compatibility alias for `references`

### Group level

- `group`: required group title
- `items`: required list of entries in that group

### Item level

- `keys`: optional shortcut chord or list of shortcut chords such as `Ctrl+P`
- `keys_by_os`: optional OS-specific key overrides keyed by `macos`, `windows`, and `linux`
- `label`: required primary text
- `value`: optional secondary text
- `command`: optional command text, rendered in a command block
- `notes`: optional descriptive text
- `url`: optional supporting link
- `search_terms`: optional list of extra search-only aliases

## How Matching Works

JustPeek first tries to match a file by process name. You can further narrow that process match with either:

- `title_pattern` for regex matching
- `title_contains` for a simpler case-insensitive substring match

If both are present, both must match.

Important:

- `title_pattern` and `title_contains` do not create an automatic match by themselves.
- They only refine a file that already has `process`.
- If `process` is omitted, the file is picker-only even when a window title would otherwise match.

Example:

```yaml
name: GitHub in Browser
process:
  - firefox
  - chromium
title_pattern: GitHub
references:
  - group: Pull Requests
    items:
      - label: Create review
        notes: Use the Review changes button
```

This helps when one app is used for many contexts and you only want the file to appear for a specific kind of window.

Simpler example:

```yaml
name: GitHub in Browser
process: firefox
title_contains: pull request
references:
  - group: Pull Requests
    items:
      - label: Create review
```

If a file has no `process`, it is treated as picker-only content. It can still be opened manually from the picker, but it will not be selected automatically from the active window. In that case, `title_pattern` and `title_contains` have no effect on auto-selection.

## How Picker Grouping Works

If you set a top-level `group`, JustPeek uses it to group files in the manual reference picker.

Example:

```yaml
name: Git Reference
group: CLI
references:
  - group: Rollback
    items:
      - label: Revert a commit
        command: git revert <commit>
```

If `group` is omitted, the file goes into `Ungrouped`.

## How to Choose `process`

You can write `process` as either a single string or a list.

Single process:

```yaml
process: code
```

Multiple variants:

```yaml
process:
  - code
  - Code
```

Start with the process name you expect the app to use. If matching is inconsistent across environments, include known variants.

Example:

```yaml
process:
  - org.wezfurlong.wezterm
  - wezterm-gui
  - wezterm
```

Keep the list focused. Only add variants that you know are real matches for the same app.

If the file is not meant to contextually match any app, omit `process` entirely. Do not rely on `title_pattern` or `title_contains` alone for contextual matching, because they only refine files that already declare `process`.

## How to Structure Good Entries

A good entry is short, scannable, and searchable.

### Good shortcut entry

```yaml
- keys: Ctrl+Shift+P
  label: Command Palette
  notes: Opens the action menu
```

### Good OS-specific shortcut entry

```yaml
- label: Quick Open
  keys: Ctrl+P
  keys_by_os:
    macos: Cmd+P
```

### Good command entry

```yaml
- label: Revert a commit
  command: git revert <commit>
  notes: Safe for shared history
  url: https://git-scm.com/docs/git-revert
```

### Good people entry

```yaml
- label: Sam Patel
  value: SRE
  notes: Escalate incident-related questions
```

### Good mixed entry

```yaml
- label: Revert a commit
  value: Safe for shared history
  command: git revert <commit>
  url: https://git-scm.com/docs/git-revert
```

### Good multi-line notes entry

```yaml
- label: Incident handoff
  notes: |
    Check the dashboard first.
    Then review the most recent deploy.
    Escalate if error rate is still rising.
```

### Good multi-line command entry

```yaml
- label: Release steps
  command: |
    git tag -a v1.2.3 -m "Release v1.2.3"
    git push --tags
```

## Authoring Tips

### Keep labels short

The `label` should be the thing you want to scan for quickly. Put extra explanation in `notes`.

### Use `value` for the most important secondary text

Use `value` for plain secondary text such as roles, short descriptions, or guidance.

For people references, `value` can be a role or team.

### Use `command` for CLI-style content

`command` is distinct from `value`.

Anything defined under `command` renders in a command block in the panel. `value` does not.

If you want visible line breaks in the command block, write `command` as a YAML block with `|`. JustPeek preserves those line breaks when rendering.

### Use `url` for source material

If an entry has a canonical doc page, runbook, or profile, store it in `url`.

Example:

```yaml
- label: Revert a commit
  command: git revert <commit>
  url: https://git-scm.com/docs/git-revert
```

In the panel, entries with a `url` show an `Open` action that launches the link externally.

### Use `notes` for context, not duplication

Good `notes` add useful guidance.

If you want visible line breaks in the panel, write `notes` as a YAML block with `|`. JustPeek preserves those line breaks when rendering.

Weak:

```yaml
- label: Quick Open
  notes: Quick Open
```

Better:

```yaml
- label: Quick Open
  notes: Opens a file by name
```

### Use `search_terms` for synonyms

If people may search for an entry using a nickname or abbreviation, put those terms in `search_terms` instead of stuffing them into `notes`.

Example:

```yaml
- label: Command Palette
  keys: Ctrl+Shift+P
  search_terms:
    - actions
    - cmd pal
    - palette
```

`search_terms` improves filtering without changing what is visibly rendered in the panel.

### Use `keys_by_os` for OS-specific variants

If the same action has different bindings on different platforms, keep one item and use `keys_by_os`.

Example:

```yaml
- label: Quick Open
  keys: Ctrl+P
  keys_by_os:
    macos: Cmd+P
```

`keys` acts as the fallback. JustPeek uses:

1. `keys_by_os.<preferred or current OS>` when present
2. otherwise `keys`

Fully explicit example:

```yaml
- label: Command Palette
  keys_by_os:
    macos: Cmd+Shift+P
    windows: Ctrl+Shift+P
    linux: Ctrl+Shift+P
```

### Group by how you search

Groups should match how you think about the content.

Good groups:

- Navigation
- Editing
- Rollback
- Product
- Platform

Avoid groups that are too broad unless the file is very small.

## Example Files

### Editor shortcuts

```yaml
name: VS Code
process:
  - code
  - Code
references:
  - group: Navigation
    items:
      - label: Quick Open
        keys: Ctrl+P
        keys_by_os:
          macos: Cmd+P
      - label: Command Palette
        keys_by_os:
          macos: Cmd+Shift+P
          windows: Ctrl+Shift+P
          linux: Ctrl+Shift+P
  - group: View
    items:
      - keys: Ctrl+B
        label: Toggle Sidebar
```

### CLI reference

```yaml
name: Git Reference
references:
  - group: Tagging
    items:
      - label: Annotated tag
        value: git tag -a v1.2.3 -m "Release v1.2.3"
      - label: Push tags
        value: git push --tags
```

### Team directory

```yaml
name: Team Reference
references:
  - group: Product
    items:
      - label: Jordan Lee
        value: Product lead
        notes: Owns roadmap sign-off
  - group: Platform
    items:
      - label: Sam Patel
        value: SRE
        notes: Escalate incident-related questions
```

## Common Mistakes

### Empty `references`

If there are no groups or no items, the file is ignored.

### Overusing regex in `title_pattern`

Use `title_pattern` only when you need regex behavior. If a simple phrase match is enough, prefer `title_contains`.

### Using one huge group

If a file has many entries, splitting them into meaningful groups makes the panel much easier to scan.

### Cramming search aliases into `notes`

Use `search_terms` for aliases instead. It keeps `notes` readable and still improves filtering.

## Recommended Style

If you want a simple rule set to follow:

1. Start with `name`, `process`, and `references`.
2. Add `title_pattern` only when one app needs multiple contexts.
3. Keep `label` short.
4. Put roles or short secondary text in `value`.
5. Put CLI-style content in `command`.
6. Put explanation in `notes`.
7. Group entries by how you naturally look them up.

## Proposed Next Improvements

The current schema works, but a few additions would make it easier to author:

- optional `process` for picker-only files
- `process` as either a string or a list
- `keys` as either a string or a list
- `search_terms` for better filtering
- `title_contains` as a simpler alternative to `title_pattern`
- `url` for docs or runbooks

See the schema proposal document for the full recommendation and rollout order.
