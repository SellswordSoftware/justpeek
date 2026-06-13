## Project Review: Current JustPeek State

**Scope**: Whole-project review based on `docs/context.md` plus the current frontend, backend, config, watcher, styling, and recent cleanup/testing work.

**Verification**:
- `npm test` passed
- `npm run typecheck` passed
- `cargo test --manifest-path src-tauri/Cargo.toml scanner` passed
- `cargo check --manifest-path src-tauri/Cargo.toml` passed

---

### Resolved Since Initial Review

#### Fixed: Saved theme now applies to the panel window

**Files**:
- `src/main.js`
- `src/settings.js`
- `src/styles/app.css`

The panel now loads the saved theme before rendering, settings changes propagate immediately across windows, and the app-shell gradient has both dark and light variants.

#### Fixed: The parallel scaffold frontend path was removed

**Files**:
- `src/main.js`
- `src/index.html`
- `src/styles/app.css`
- `src/app/create-app.js` (removed)
- `src/app/lifecycle.js` (removed)
- `src/pages/panel/panel-page.js` (removed)

The frontend now has one authoritative runtime path: the Tauri event-driven app. The old sample-panel/browser scaffold is gone, replaced with a minimal non-Tauri fallback message.

#### Fixed: Tauri command naming is now centralized

**Files**:
- `src/runtime/tauri.js`
- `src/main.js`
- `src/settings.js`
- `src/panel.js`

The frontend no longer mixes raw command names, fallback arrays, and ad hoc event constants. Backend command naming and shared events now live behind one runtime wrapper surface.

#### Fixed: Stale shell/CSS leftovers were pruned

**Files**:
- `src/index.html`
- `src/styles/app.css`
- `src/runtime/tauri.js`
- `src/main.js`
- `src/panel.js`

Placeholder shell markup, dead selectors, duplicate picker helpers, and an unused runtime export were removed.

#### Fixed: The project now has a runnable test entry point and meaningful coverage

**Files**:
- `package.json`
- `src/runtime/fuzzy.test.js`
- `src/runtime/tauri.test.js`
- `src-tauri/src/scanner.rs`

`npm test` now exists and covers the frontend Tauri wrapper surface. Rust unit tests now cover scanner normalization, contextual lookup, title fallback, picker sorting, and picker lookup.

#### Fixed: Debug diagnostics are enabled in development but compiled out of release builds

**Files**:
- `src-tauri/src/lib.rs`

`debug_log()` now emits trace lines only under `debug_assertions` and compiles to a no-op in release builds.

---

### Remaining Findings

### [LOW] Cleanup Opportunity: `@tauri-apps/api` appears unused

**File**: `package.json`

**Issue**: The dependency is declared but not imported anywhere in `src/`.

**Why it matters**: It is small, but removing unused dependencies reduces ambiguity about the runtime surface and avoids carrying packages that are no longer part of the implementation.

**Suggestion**: Remove `@tauri-apps/api` if there is no planned near-term use for it.

### [LOW] Operability: Debug traces are available in dev builds, but there is still no structured logging path

**Files**:
- `src-tauri/src/lib.rs`
- `src-tauri/src/detection/mod.rs`

**Issue**: The immediate no-op problem is fixed, but diagnostics still go to `stderr` only in debug builds.

**Why it matters**: This is adequate for development, but it is still limited if you later want richer field reports, persisted logs, or platform-specific troubleshooting without attaching a debugger.

**Suggestion**: If operational support becomes important, move these traces to a structured logging sink or Tauri logging plugin behind an explicit debug/verbose flag.

### [LOW] Testing Gap: Frontend behavior above the runtime wrapper is still mostly manual

**Files**:
- `src/main.js`
- `src/panel.js`
- `src/settings.js`

**Issue**: The non-UI logic now has better coverage, but view-level behavior such as theme bootstrapping at mount time, picker-mode transitions, and keyboard flows still relies on manual validation.

**Why it matters**: The riskiest integration seams have improved, but some user-facing interaction regressions could still slip through.

**Suggestion**: Add a small next layer of tests around pure/stateful frontend behavior before considering browser automation.

---

### Summary

| Severity | Count |
|----------|-------|
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 3 |

### Recommended Actions

1. Remove `@tauri-apps/api` if it is truly unused.
2. Decide whether current debug-only `stderr` tracing is sufficient or whether you want structured logging.
3. Add one more layer of frontend behavior tests around panel/settings state transitions.
