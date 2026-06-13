# JustPeek Reference File Schema Proposal

## Audience

This document is for maintainers and contributors deciding how the reference-file format should evolve.

## Goal

After reading this, you should be able to:

- understand the current schema and its constraints
- evaluate a concrete next-version schema
- decide which additions are safe to implement first

## Current Schema

Today a reference file is a YAML document with these top-level fields:

```yaml
name: VS Code
process:
  - code
  - Code
title_pattern: "workspace"
references:
  - group: Navigation
    items:
      - keys: Ctrl+P
        label: Quick Open
        notes: Opens a file by name
```

Current supported fields:

- `name`: display name shown in the panel and picker
- `process`: list of process names that may contextually match
- `title_pattern`: optional regex checked against the active window title
- `title_contains`: optional case-insensitive window-title substring matcher
- `references`: canonical top-level list of groups
- `shortcuts`: compatibility alias for `references`
- `group`: section title inside `references`
- `items`: list of entries inside a group
- `keys`: optional shortcut chord string
- `label`: primary display text
- `value`: optional secondary text, often a command or role
- `command`: alias for `value`
- `notes`: optional tertiary description

## Current Strengths

- Small and easy to learn
- Flexible enough for shortcuts, CLI references, and people references
- Good fit for the current panel UI
- Backward-compatible alias support already exists for `shortcuts`

## Current Friction

### Manual-only references must pretend to be contextual

The manual picker is now a first-class flow, but the schema still assumes every file needs a `process` list. That pushes authors to invent fake app matches for reference packs that are really just general-purpose lookup data.

### Shortcut data is too narrow for real-world key bindings

`keys` is a single string. That works for one chord, but it is awkward for:

- Mac and Windows variants
- more than one valid binding
- sequences such as `g g`

### Search synonyms have no explicit home

The current filter only sees `keys`, `label`, `value`, and `notes`. Authors who want better search are pushed to overload `notes` with extra words.

### `title_pattern` is powerful but heavier than many users need

Regex is useful for advanced matching, but a large share of files could get by with a simple substring rule.

### The structure is slightly verbose for short files

For small reference packs, `references -> group -> items` can feel heavier than necessary.

## Proposed Schema Direction

The proposal below is intentionally incremental. It keeps existing files valid and adds the most useful authoring improvements first.

## Proposed vNext Schema

```yaml
name: VS Code
process:
  - code
  - Code
title_contains: workspace
tags: [editor, shortcuts]
references:
  - group: Navigation
    items:
      - label: Quick Open
        keys:
          - Ctrl+P
          - Cmd+P
        search_terms:
          - open file
          - fuzzy open
        notes: Opens a file by name

      - label: Command Palette
        keys: Ctrl+Shift+P
        search_terms:
          - actions
          - cmd pal

  - group: Git
    items:
      - label: Revert a commit
        command: git revert <commit>
        notes: Safe for shared history
        url: https://git-scm.com/docs/git-revert
```

## Proposed Additions

### 1. Optional `process`

Allow a file with no `process` field, or an empty one, to exist as picker-only content.

Example:

```yaml
name: Git Reference
references:
  - group: Rollback
    items:
      - label: Revert a commit
        command: git revert <commit>
```

Behavior:

- file appears in the manual picker
- file never wins contextual process matching

This is the highest-value simplification.

### 2. `process` may be a string or a list

Allow both:

```yaml
process: code
```

and:

```yaml
process:
  - code
  - Code
```

This reduces noise for the common single-process case.

### 3. `title_contains` as a lightweight matcher

Add a simple substring matcher alongside `title_pattern`.

Example:

```yaml
title_contains: pull request
```

Suggested behavior:

- `title_contains` performs a case-insensitive substring check
- `title_pattern` remains available for advanced regex cases
- if both are present, both must match

This gives authors an easier default without removing regex power.

### 4. `keys` may be a string or a list of strings

Allow both:

```yaml
keys: Ctrl+P
```

and:

```yaml
keys:
  - Ctrl+P
  - Cmd+P
```

Suggested rendering:

- one item row
- multiple key variants shown inline or stacked

This improves cross-platform authoring without requiring duplicate entries.

Status:

- implemented

### 5. `search_terms` on items

Add explicit search-only synonyms.

Example:

```yaml
- label: Command Palette
  keys: Ctrl+Shift+P
  search_terms:
    - actions
    - palette
    - cmd pal
```

This keeps `notes` readable while making fuzzy search more useful.

Status:

- implemented

### 6. `command` as a clearer alias for `value`

For command references, `value` is generic. `command` is clearer.

Suggested behavior:

- support `command` as an alias for `value`
- prefer documenting `command` in examples for CLI-style entries
- keep `value` for non-command content such as roles or short descriptions

Status:

- implemented

### 7. `url` on items

Add an optional link field for docs, runbooks, or profiles.

Example:

```yaml
- label: Revert a commit
  command: git revert <commit>
  url: https://git-scm.com/docs/git-revert
```

This is useful even before the UI makes links clickable, because it preserves source material in the data model.

Status:

- implemented

### 8. `tags` on files or items

Tags are not essential for the current UI, but they create room for:

- better search
- future filtering
- higher-level grouping

Example:

```yaml
tags: [git, cli]
```

or:

```yaml
- label: Revert a commit
  tags: [safe, rollback]
```

## Optional Future Simplifications

These are worth considering later, but they are less urgent than the changes above.

### Flat `items` without explicit groups

Possible shorthand:

```yaml
name: Team Reference
items:
  - label: Jordan Lee
    value: Product lead
```

This would be internally normalized into a default group.

This is user-friendly, but it adds another normalization path and is not necessary for the next iteration.

### `platform` on files or items

Useful for shortcut-heavy data:

```yaml
platform: [mac, windows]
```

or:

```yaml
- label: Quick Open
  keys: Cmd+P
  platform: mac
```

This is valuable, but it becomes more compelling once multi-key support exists.

### `match_priority` on files

If title fallback grows more common, an explicit priority can resolve ambiguity:

```yaml
match_priority: 50
```

This is useful, but only if real matching conflicts become common enough to justify extra schema weight.

## Recommended Implementation Order

### Phase 1

- make `process` optional
- allow `process` to be a string or list
- add `search_terms`

These are simple, high-value improvements with minimal UI impact.

### Phase 2

- allow `keys` to be a string or list
- add `title_contains`
- add `command` as an alias for `value`

These improve authoring clarity and cross-platform ergonomics.

### Phase 3

- add `url`
- add `tags`
- consider `platform`

These prepare the model for future UI upgrades.

## Backward Compatibility Guidance

The next schema should preserve existing files unchanged.

Compatibility expectations:

- existing `references` files remain valid
- existing `shortcuts` alias remains valid
- existing `keys` strings remain valid
- existing `value` fields remain valid
- new fields are optional and additive

## Recommendation

The most pragmatic next step is:

1. allow picker-only files by making `process` optional
2. support `search_terms`
3. support `process` as either a string or a list
4. add `keys` list support
5. add `title_contains`

That sequence keeps the format easy to explain while removing the biggest authoring friction.
