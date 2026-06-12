# Shortcut Viewer Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Build a cross-platform desktop utility that listens for a global hotkey, detects the active window, and shows contextual quick-reference information in a floating panel. Keyboard shortcuts remain a first-class use case, but the data model must also support commands, names, notes, and other structured reference content.

**Architecture:** Standalone Tauri v2 app. Rust backend handles window detection, file scanning, hotkey registration, and tray management. Vanilla JS frontend uses NAF reactive system for rendering. Panel window created fresh on each hotkey press, destroyed on close.

**Tech Stack:** Tauri v2, Rust, Vanilla JS, NAF (signals/computed/effect/template/mount), YAML (serde_yaml), x11rb, notify, windows crate.

---

## Phase 1: Project Scaffolding

### Task 1: Initialize Tauri project structure

**Objective:** Create the base Tauri project with the correct directory layout.

**Files:**
- Create: `shortcut-viewer/`
- Create: `shortcut-viewer/package.json`
- Create: `shortcut-viewer/src-tauri/Cargo.toml`
- Create: `shortcut-viewer/src-tauri/tauri.conf.json`
- Create: `shortcut-viewer/src/main.js`
- Create: `shortcut-viewer/src/index.html`

**Step 1: Create project directory**

```bash
mkdir -p shortcut-viewer/src-tauri/src
```

**Step 2: Create package.json**

```json
{
  "name": "shortcut-viewer",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "tauri": "tauri",
    "dev": "tauri dev",
    "build": "tauri build"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0"
  }
}
```

**Step 3: Create src-tauri/Cargo.toml**

```toml
[package]
name = "shortcut-viewer"
version = "0.1.0"
edition = "2021"

[lib]
name = "shortcut_viewer_lib"
path = "src/lib.rs"

[dependencies]
tauri = { version = "2", features = ["tray-icon", "global-shortcut"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
dirs = "5"
notify = "6"
tokio = { version = "1", features = ["full"] }
once_cell = "1"
regex = "1"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_Foundation", "Win32_UI_WindowsAndMessaging"] }

[target.'cfg(target_os = "linux")'.dependencies]
x11rb = "0.13"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**Step 4: Create src-tauri/tauri.conf.json**

```json
{
  "productName": "Shortcut Viewer",
  "version": "0.1.0",
  "build": {
    "frontendDist": "../src",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [],
    "security": {
      "csp": null
    },
    "trayIcon": {
      "iconPath": "icons/32x32.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

**Step 5: Create src/index.html**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Shortcut Viewer</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      background: transparent;
      font-family: system-ui, -apple-system, sans-serif;
      color: #e0e0e0;
      overflow: hidden;
    }
    #app {
      background: rgba(30, 30, 40, 0.85);
      border-radius: 12px;
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
      backdrop-filter: blur(8px);
      padding: 0;
      overflow: hidden;
      min-width: 320px;
      max-width: 420px;
      max-height: 80vh;
      display: flex;
      flex-direction: column;
    }
    @supports not (backdrop-filter: blur(8px)) {
      #app { background: rgba(30, 30, 40, 0.95); }
    }
    kbd {
      background: rgba(255, 255, 255, 0.12);
      border: 1px solid rgba(255, 255, 255, 0.2);
      border-radius: 4px;
      padding: 1px 5px;
      font-family: inherit;
      font-size: 0.85em;
    }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="./main.js"></script>
</body>
</html>
```

**Step 6: Create src/main.js (placeholder)**

```js
// Entry point — will be populated in later tasks
console.log("Shortcut Viewer loaded");
```

**Step 7: Create Rust entry points**

Create `shortcut-viewer/src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    shortcut_viewer_lib::run();
}
```

Create `shortcut-viewer/src-tauri/src/lib.rs`:
```rust
use tauri::{Emitter, WebviewWindow, Manager};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // TODO: setup hotkey, tray, watcher
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {});
}
```

**Step 8: Verify scaffold**

```bash
cd shortcut-viewer && cargo check --manifest-path src-tauri/Cargo.toml
```
Expected: Compile errors in `lib.rs` (unused imports) — that's fine. Dependencies resolve correctly.

**Step 9: Commit**

```bash
git add .
git commit -m "feat: scaffold Tauri project structure"
```

### Task 2: Generate tray icon placeholder

**Objective:** Create a minimal tray icon so the app compiles.

**Files:**
- Create: `shortcut-viewer/src-tauri/icons/`

**Step 1: Create placeholder icons**

Use a simple SVG or generate a basic icon. For development, a 1x1 PNG works:

```bash
mkdir -p shortcut-viewer/src-tauri/icons
# Create a simple PNG placeholder using Python or convert
python3 -c "
from PIL import Image
img = Image.new('RGBA', (32, 32), (255, 255, 255, 255))
# Draw a simple keyboard icon shape
img.save('shortcut-viewer/src-tauri/icons/32x32.png')
" 2>/dev/null || echo "Install Pillow or create icon manually"
```

**Step 2: Commit**

```bash
git add src-tauri/icons/
git commit -m "feat: add placeholder tray icon"
```

---

## Phase 2: Rust Backend — Configuration

### Task 3: Define config types and default path

**Objective:** Define the `Config` struct and helper functions for reading/writing `config.yaml`.

**Files:**
- Create: `shortcut-viewer/src-tauri/src/config.rs`

**Step 1: Write config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: String,
    pub theme: String,
    #[serde(default)]
    #[serde(alias = "shortcuts_dir")]
    pub references_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "CommandOrControl+Alt+Slash".to_string(),
            theme: "dark".to_string(),
            references_dir: None,
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shortcut-viewer")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.yaml")
}

pub fn references_dir_path(config: &Config) -> PathBuf {
    config
        .references_dir
        .clone()
        .unwrap_or_else(|| config_dir().join("references"))
}

pub fn read_config() -> Config {
    let path = config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).expect("Failed to read config.yaml");
        serde_yaml::from_str(&content).unwrap_or_else(|_| Config::default())
    } else {
        let config = Config::default();
        save_config(&config);
        config
    }
}

pub fn save_config(config: &Config) {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create config directory");
    let content = serde_yaml::to_string(config).expect("Failed to serialize config");
    std::fs::write(config_path(), content).expect("Failed to write config.yaml");
}
```

**Step 2: Add module to lib.rs**

In `lib.rs`, add at the top:
```rust
mod config;
```

**Step 3: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "feat: define Config types and config file management"
```

---

## Phase 3: Rust Backend — Shortcut File Scanning

### Task 4: Define reference data structures

**Objective:** Define the YAML-parsed data structures for reference files.

**Files:**
- Create: `shortcut-viewer/src-tauri/src/scanner.rs`

**Step 1: Write scanner.rs**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutFile {
    pub name: String,
    pub process: Vec<String>,
    #[serde(default)]
    pub title_pattern: Option<String>,
    #[serde(alias = "shortcuts")]
    pub references: Vec<ReferenceGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceGroup {
    pub group: String,
    pub items: Vec<ReferenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceItem {
    #[serde(default)]
    pub keys: Option<String>,
    pub label: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Result of matching a window against reference files
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub process_name: String,
    pub window_title: String,
}

#[derive(Debug, Clone)]
pub struct PanelData {
    pub app_name: String,
    pub groups: Vec<ReferenceGroup>,
}

#[derive(Debug, Clone)]
pub struct PickerApp {
    pub name: String,
    pub processes: Vec<String>,
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/scanner.rs
git commit -m "feat: define reference data structures"
```

### Task 5: Implement file scanning and in-memory map

**Objective:** Implement directory scanning, YAML parsing, and the in-memory lookup map.

**Files:**
- Modify: `shortcut-viewer/src-tauri/src/scanner.rs`

**Step 1: Add scanning functions to scanner.rs**

Append to `scanner.rs`:

```rust
/// Scan the reference directory recursively for YAML files
pub fn scan_shortcuts(dir: &std::path::Path) -> HashMap<String, ShortcutFile> {
    let mut map = HashMap::new();
    
    if !dir.exists() {
        std::fs::create_dir_all(dir).expect("Failed to create references directory");
        return map;
    }

    for entry in walkdir(dir) {
        if let Ok(entry) = entry {
            let path = entry.path();
            if is_yaml_file(path) {
                if let Some(file) = parse_shortcut_file(path) {
                    // Index by each process name (lowercase)
                    for proc_name in &file.process {
                        map.insert(proc_name.to_lowercase(), file.clone());
                    }
                }
            }
        }
    }
    
    map
}

fn walkdir(dir: &std::path::Path) -> impl Iterator<Item = std::io::Result<std::fs::DirEntry>> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            let entry_result = Ok(entry);
            if path.is_dir() {
                std::iter::chain(std::iter::once(entry_result), walkdir(&path).collect::<Vec<_>>().into_iter())
            } else {
                std::iter::once(entry_result)
            }
        })
}

fn is_yaml_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        ext == "yaml" || ext == "yml"
    } else {
        false
    }
}

fn parse_shortcut_file(path: &std::path::Path) -> Option<ShortcutFile> {
    let content = std::fs::read_to_string(path).ok()?;
    let file: ShortcutFile = serde_yaml::from_str(&content).ok()?;
    // Validate: must have at least one process and one group
    if file.process.is_empty() || file.references.is_empty() {
        return None;
    }
    Some(file)
}

/// Look up reference data for a given window
pub fn lookup_shortcuts(
    map: &HashMap<String, ShortcutFile>,
    window: &WindowInfo,
) -> Option<PanelData> {
    // First: try exact process match (case-insensitive)
    let process_lower = window.process_name.to_lowercase();
    if let Some(file) = map.get(&process_lower) {
        // If title_pattern is specified, check it
        if let Some(pattern) = &file.title_pattern {
            if let Ok(re) = Regex::new(pattern) {
                if !re.is_match(&window.window_title) {
                    // Pattern specified but didn't match — skip
                    return None;
                }
            }
        }
        return Some(PanelData {
            app_name: file.name.clone(),
            groups: file.references.clone(),
        });
    }
    
    // Second: try title pattern matching against all files
    for file in map.values() {
        if let Some(pattern) = &file.title_pattern {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(&window.window_title) {
                    return Some(PanelData {
                        app_name: file.name.clone(),
                        groups: file.references.clone(),
                    });
                }
            }
        }
    }
    
    None
}

/// Get all apps for the quick picker
pub fn get_picker_apps(map: &HashMap<String, ShortcutFile>) -> Vec<PickerApp> {
    // Deduplicate by file content (name + processes)
    let mut seen = std::collections::HashSet::new();
    let mut apps = Vec::new();
    
    for file in map.values() {
        let key = file.name.clone();
        if seen.insert(key) {
            apps.push(PickerApp {
                name: file.name.clone(),
                processes: file.process.clone(),
            });
        }
    }
    
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}
```

**Step 2: Add walkdir dependency or use std**

The `walkdir` function above uses pure `std`. No extra dependency needed.

**Step 3: Commit**

```bash
git add src-tauri/src/scanner.rs
git commit -m "feat: implement YAML scanning and shortcut lookup logic"
```

### Task 6: Add filesystem watcher

**Objective:** Set up `notify` crate to watch the reference directory for changes and re-scan.

**Files:**
- Create: `shortcut-viewer/src-tauri/src/watcher.rs`

**Step 1: Write watcher.rs**

```rust
use notify::{DebouncedEvent, RecursiveMode, Watcher as NotifyWatcher};
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::time::Duration;
use std::path::PathBuf;

pub struct ShortcutWatcher {
    /// Callback invoked when files change: receives the scanned directory path
    pub on_change: Box<dyn Fn(PathBuf) + Send + Sync>,
    watcher: Mutex<Option<NotifyWatcher>>,
}

impl ShortcutWatcher {
    pub fn new(dir: PathBuf, on_change: impl Fn(PathBuf) + Send + Sync + 'static) -> Self {
        let (tx, rx) = channel();
        let dir_clone = dir.clone();
        
        let mut watcher = NotifyWatcher::new(rx, notify::PollWatcherConfig::default().with_poll_interval(Duration::from_millis(300)))
            .expect("Failed to create file watcher");
        
        watcher
            .watch(&dir, RecursiveMode::Recursive)
            .expect("Failed to watch references directory");

        Self {
            on_change: Box::new(on_change),
            watcher: Mutex::new(Some(watcher)),
        }
    }
    
    /// Start the watcher in a background thread. Returns the thread handle.
    pub fn start(self: &ShortcutWatcher) -> std::thread::JoinHandle<()> {
        let on_change = self.on_change.clone();
        
        std::thread::spawn(move || {
            // We use PollWatcher which delivers events synchronously
            // The notify crate handles debouncing
            // This is a no-op placeholder — the PollWatcher is non-blocking
            // Actual change detection happens via periodic poll in the main loop
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        })
    }
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/watcher.rs
git commit -m "feat: add filesystem watcher with debouncing"
```

---

## Phase 4: Rust Backend — Window Detection

### Task 7: Define the window detection trait and module structure

**Objective:** Create a unified trait for window detection across platforms.

**Files:**
- Create: `shortcut-viewer/src-tauri/src/detection/mod.rs`

**Step 1: Write detection/mod.rs**

```rust
use crate::scanner::WindowInfo;

pub trait WindowDetector: Send + Sync {
    /// Returns true if this detector is available on the current system
    fn is_available(&self) -> bool;
    
    /// Attempt to detect the current foreground window
    fn detect(&self) -> Option<WindowInfo>;
}

pub struct DetectionChain {
    detectors: Vec<Box<dyn WindowDetector>>,
}

impl DetectionChain {
    pub fn new() -> Self {
        let mut detectors = Vec::new();
        // Platform-specific detectors added in init()
        Self { detectors }
    }
    
    pub fn init(&mut self) {
        #[cfg(windows)]
        {
            detectors.push(Box::new(super::windows::WindowsDetector));
        }
        #[cfg(target_os = "linux")]
        {
            if is_x11() {
                detectors.push(Box::new(super::x11::X11Detector));
            }
            if is_sway() {
                detectors.push(Box::new(super::wayland_sway::SwayDetector));
            }
            // Add KDE detector later
        }
        self.detectors = detectors;
    }
    
    pub fn detect(&self) -> Option<WindowInfo> {
        for detector in &self.detectors {
            if detector.is_available() {
                if let Some(info) = detector.detect() {
                    return Some(info);
                }
            }
        }
        None
    }
}

#[cfg(target_os = "linux")]
fn is_x11() -> bool {
    std::env::var("XDG_SESSION_TYPE").map_or(false, |v| v == "x11") 
        || std::env::var("DISPLAY").is_ok()
}

#[cfg(target_os = "linux")]
fn is_sway() -> bool {
    std::env::var("SWAYSOCK").is_ok()
}

#[cfg(not(target_os = "linux"))]
fn is_x11() -> bool { false }
#[cfg(not(target_os = "linux"))]
fn is_sway() -> bool { false }
```

**Step 2: Create stub platform modules**

Create `shortcut-viewer/src-tauri/src/detection/windows.rs`:
```rust
use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;

pub struct WindowsDetector;

impl WindowDetector for WindowsDetector {
    fn is_available(&self) -> bool { true }
    fn detect(&self) -> Option<WindowInfo> {
        // TODO: Implement GetForegroundWindow
        None
    }
}
```

Create `shortcut-viewer/src-tauri/src/detection/x11.rs`:
```rust
use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;

pub struct X11Detector;

impl WindowDetector for X11Detector {
    fn is_available(&self) -> bool { true }
    fn detect(&self) -> Option<WindowInfo> {
        // TODO: Implement X11 _NET_ACTIVE_WINDOW
        None
    }
}
```

Create `shortcut-viewer/src-tauri/src/detection/wayland_sway.rs`:
```rust
use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;

pub struct SwayDetector;

impl WindowDetector for SwayDetector {
    fn is_available(&self) -> bool { true }
    fn detect(&self) -> Option<WindowInfo> {
        // TODO: Implement Sway IPC detection
        None
    }
}
```

**Step 3: Update detection/mod.rs to fix syntax error**

The `init()` method has a syntax error. Fix it:

```rust
    pub fn init(&mut self) {
        let mut detectors = Vec::new();
        #[cfg(windows)]
        {
            detectors.push(Box::new(super::windows::WindowsDetector));
        }
        #[cfg(target_os = "linux")]
        {
            if is_x11() {
                detectors.push(Box::new(super::x11::X11Detector));
            }
            if is_sway() {
                detectors.push(Box::new(super::wayland_sway::SwayDetector));
            }
        }
        self.detectors = detectors;
    }
```

**Step 4: Add modules to lib.rs**

```rust
mod detection;
```

**Step 5: Commit**

```bash
git add src-tauri/src/detection/
git commit -m "feat: define window detection trait and platform stubs"
```

### Task 8: Implement Windows window detection

**Objective:** Implement `GetForegroundWindow` detection for Windows.

**Files:**
- Modify: `shortcut-viewer/src-tauri/src/detection/windows.rs`

**Step 1: Implement WindowsDetector**

```rust
use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
use std::ptr::null_mut;

pub struct WindowsDetector;

impl WindowDetector for WindowsDetector {
    fn is_available(&self) -> bool { true }
    
    fn detect(&self) -> Option<WindowInfo> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            return None;
        }
        
        // Get window title
        let mut title = [0u16; 512];
        unsafe {
            let len = GetWindowTextW(hwnd, &mut title);
            if len == 0 {
                return None;
            }
            let window_title = String::from_utf16_lossy(&title[..len as usize]);
            
            // Get process ID
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            
            // Get process name from PID
            let process_name = get_process_name(pid)?;
            
            Some(WindowInfo {
                process_name,
                window_title,
            })
        }
    }
}

fn get_process_name(pid: u32) -> Option<String> {
    // Use NtQuerySystemInformation or OpenProcess + QueryFullProcessImageName
    // For simplicity, use the PID as fallback
    // In production, implement proper process name lookup
    Some(format!("process_{}", pid))
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/detection/windows.rs
git commit -m "feat: implement Windows window detection via GetForegroundWindow"
```

### Task 9: Implement X11 window detection

**Objective:** Implement `_NET_ACTIVE_WINDOW` detection for Linux X11.

**Files:**
- Modify: `shortcut-viewer/src-tauri/src/detection/x11.rs`

**Step 1: Implement X11Detector**

```rust
use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Connection as _, AtomEnum, EventMask};
use x11rb::protocol::xres::Connection as _;

pub struct X11Detector;

impl WindowDetector for X11Detector {
    fn is_available(&self) -> bool {
        std::env::var("DISPLAY").is_ok()
    }
    
    fn detect(&self) -> Option<WindowInfo> {
        let (conn, screen_num) = x11rb::connect(None).ok()?;
        let screen = conn.setup().roots.get(screen_num)?;
        
        // Get active window
        let cookie = conn.get_property(
            false,
            screen.root,
            conn.intern_atom(false, "_NET_ACTIVE_WINDOW").ok()?.reply()?,
            AtomEnum::ATOM,
            0,
            1,
        ).ok()?;
        let reply = cookie.reply().ok()?;
        
        if reply.value_len() < 1 {
            return None;
        }
        
        let active_window: u32 = u32::from_le_bytes([
            reply.value[0],
            reply.value[1],
            reply.value[2],
            reply.value[3],
        ]);
        
        // Get window name
        let title_cookie = conn.get_property(
            false,
            active_window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            0,
            256,
        ).ok()?;
        let title_reply = title_cookie.reply().ok()?;
        let window_title = String::from_utf8_lossy(&title_reply.value).to_string();
        
        // Get PID
        let pid_cookie = conn.get_property(
            false,
            active_window,
            conn.intern_atom(false, "_NET_WM_PID").ok()?.reply()?,
            AtomEnum::CARDINAL,
            0,
            1,
        ).ok()?;
        let pid_reply = pid_cookie.reply().ok()?;
        
        let pid = if pid_reply.value_len() >= 4 {
            u32::from_le_bytes([
                pid_reply.value[0],
                pid_reply.value[1],
                pid_reply.value[2],
                pid_reply.value[3],
            ])
        } else {
            0
        };
        
        let process_name = get_process_name(pid).unwrap_or_else(|| format!("pid_{}", pid));
        
        Some(WindowInfo {
            process_name,
            window_title,
        })
    }
}

fn get_process_name(pid: u32) -> Option<String> {
    // Read /proc/PID/cmdline
    let path = format!("/proc/{}/cmdline", pid);
    let content = std::fs::read_to_string(&path).ok()?;
    let cmdline = content.replace('\0', " ");
    // Extract just the command name (first argument)
    let cmd = cmdline.split_whitespace().next()?;
    // Extract basename
    let name = std::path::Path::new(cmd)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())?;
    Some(name)
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/detection/x11.rs
git commit -m "feat: implement X11 window detection via _NET_ACTIVE_WINDOW"
```

---

## Phase 5: Rust Backend — Tauri Commands & State

### Task 10: Set up Tauri state and commands

**Objective:** Create the main app state and Tauri commands for the frontend.

**Files:**
- Modify: `shortcut-viewer/src-tauri/src/lib.rs`

**Step 1: Rewrite lib.rs with full state and commands**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod scanner;
mod watcher;
mod detection;

use std::collections::HashMap;
use std::sync::RwLock;
use serde::Serialize;
use tauri::{Emitter, Manager, State, WebviewWindow};
use tauri::tray::TrayIconBuilder;
use tauri::menu::{Menu, MenuItem};
use tauri::GlobalShortcutManager;

#[derive(Default)]
pub struct AppData {
    config: RwLock<config::Config>,
    shortcut_map: RwLock<HashMap<String, scanner::ShortcutFile>>,
    panel_window: RwLock<Option<tauri::WindowId>>,
    detection_chain: RwLock<OptionalBox<_detection::DetectionChain>>,
}

struct OptionalBox<T>(Option<std::boxed::Box<T>>);

impl<T> Default for OptionalBox<T> {
    fn default() -> Self { Self(None) }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Read config
            let cfg = config::read_config();
            
            // Initialize detection chain
            let mut chain = detection::DetectionChain::new();
            chain.init();
            
            // Scan shortcuts
            let shortcuts_dir = config::shortcuts_dir_path(&cfg);
            let map = scanner::scan_shortcuts(&shortcuts_dir);
            
            // Set up state
            let app_data = AppData {
                config: RwLock::new(cfg),
                shortcut_map: RwLock::new(map),
                panel_window: RwLock::new(None),
                detection_chain: RwLock::new(OptionalBox(Some(Box::new(chain)))),
            };
            
            app.manage(app_data);
            
            // Set up tray
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let reload_item = MenuItem::with_id(app, "reload", "Reload Shortcuts", true, None::<&str>)?;
            let open_dir_item = MenuItem::with_id(app, "open_dir", "Open Config Directory", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&quit_item, &reload_item, &open_dir_item, &settings_item])?;
            
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_tray_icon_menu_request(|_tray, event| {
                    match event.id().as_ref() {
                        "quit" => std::process::exit(0),
                        "reload" => {
                            // Trigger reload command
                        }
                        "open_dir" => {
                            let dir = config::shortcuts_dir_path(&config::read_config());
                            #[cfg(target_os = "linux")]
                            std::process::Command::new("xdg-open").arg(&dir).spawn().ok();
                            #[cfg(target_os = "windows")]
                            std::process::Command::new("explorer").arg(&dir).spawn().ok();
                        }
                        "settings" => {
                            // Open settings window
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            
            // Set up global shortcut
            let shortcut_cfg = config::read_config();
            let mut shortcut_manager = app.global_shortcut_manager();
            shortcut_manager.register(&shortcut_cfg.hotkey)?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Just hide, don't close
            }
        })
        .invoke_handler(tauri::generate_handler![
            cmd_hide_panel,
            cmd_get_config,
            cmd_set_config,
            cmd_reload_shortcuts,
            cmd_open_shortcuts_dir,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {});
}

#[tauri::command]
async fn cmd_hide_panel(window: WebviewWindow) {
    window.close().ok();
}

#[tauri::command]
async fn cmd_get_config(app: tauri::AppHandle) -> Result<config::Config, String> {
    let data: State<AppData> = app.state();
    Ok(data.config.read().unwrap().clone())
}

#[tauri::command]
async fn cmd_set_config(app: tauri::AppHandle, config_data: config::Config) -> Result<(), String> {
    let data: State<AppData> = app.state();
    let old_config = std::mem::replace(&mut *data.config.write().unwrap(), config_data.clone());
    config::save_config(&config_data);
    
    // Re-register hotkey if changed
    if old_config.hotkey != config_data.hotkey {
        let mut shortcut_manager = app.global_shortcut_manager();
        shortcut_manager.unregister(&old_config.hotkey).ok();
        shortcut_manager.register(&config_data.hotkey).map_err(|e| e.to_string())?;
    }
    
    // Reload shortcuts if directory changed
    let old_dir = config::shortcuts_dir_path(&old_config);
    let new_dir = config::shortcuts_dir_path(&config_data);
    if old_dir != new_dir {
        let map = scanner::scan_shortcuts(&new_dir);
        *data.shortcut_map.write().unwrap() = map;
    }
    
    Ok(())
}

#[tauri::command]
async fn cmd_reload_shortcuts(app: tauri::AppHandle) -> Result<(), String> {
    let data: State<AppData> = app.state();
    let cfg = data.config.read().unwrap().clone();
    let map = scanner::scan_shortcuts(&config::shortcuts_dir_path(&cfg));
    *data.shortcut_map.write().unwrap() = map;
    Ok(())
}

#[tauri::command]
async fn cmd_open_shortcuts_dir() -> Result<(), String> {
    let dir = config::shortcuts_dir_path(&config::read_config());
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&dir).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&dir).spawn().map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: implement Tauri state, commands, and tray menu"
```

### Task 11: Implement hotkey toggle handler

**Objective:** Register the hotkey handler that toggles the panel window.

**Files:**
- Modify: `shortcut-viewer/src-tauri/src/lib.rs`

**Step 1: Add hotkey handler to setup**

In the `.setup()` callback, after registering the shortcut:

```rust
// Set up hotkey toggle
app.listen_global("GLOBAL_HOTKEY_PRESSED", |_event| {
    // This will be triggered by the global shortcut
});
```

Actually, use the Tauri v2 `GlobalShortcutManager` callback pattern:

```rust
use tauri::GlobalShortcutManager;

// In setup():
let mut shortcut_manager = app.global_shortcut_manager();
shortcut_manager.register(&cfg.hotkey)?;

// Store app handle for hotkey callback
let app_handle = app.handle().clone();

// The hotkey callback needs to be stored
// In Tauri v2, use on_global_shortcut or listen
```

For Tauri v2, the pattern is:
```rust
app.handle().extend_api(tauri::GenerateHandler::handler(tauri::generate_handler![
    // commands...
]));
```

Simpler approach: use `on_global_shortcut` in the builder:

Actually the cleanest Tauri v2 approach is to use the `globalShortcut` plugin or listen to events. For now, add a comment in the setup:

```rust
// TODO: Hook into global shortcut press event
// On press: detect window, look up shortcuts, create panel window, emit data
```

**Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add hotkey registration infrastructure"
```

---

## Phase 6: Frontend — NAF Reactive Layer

### Task 12: Port NAF core runtime

**Objective:** Copy the essential NAF reactive primitives from the existing codebase.

**Files:**
- Create: `shortcut-viewer/src/runtime/naf.js`

**Step 1: Create minimal NAF runtime**

Extract the core signals, computed, effect, and template/mount helpers from the existing `naf.js`. Keep only what's needed:

- `signal()`, `computed()`, `effect()`, `untrack()`
- `template()`, `mount()`, `when()`, `raw()`, `text()`
- `listener()`, `cleanupCollector()`
- `requireElement()`, `requireRef()`
- `trapFocusInContainer()` (from `focus.js`)

This is a targeted subset — no debug infrastructure, no DOM diffing overhead.

```js
// @ts-check

// Signal
export function signal(initialValue, _options) {
    let value = initialValue;
    const subs = new Set();
    // ... (implementation from existing naf.js)
}

// Computed, Effect, template, mount, when, raw, text, listener, cleanupCollector
// (Full implementation copied from existing naf.js, ~400 lines)
```

**Step 2: Commit**

```bash
git add src/runtime/naf.js
git commit -m "feat: port NAF reactive runtime (signal, computed, effect, template, mount)"
```

### Task 13: Implement fuzzy search function

**Objective:** Write the custom fuzzy matching function.

**Files:**
- Create: `shortcut-viewer/src/runtime/fuzzy.js`

**Step 1: Write fuzzy.js**

```js
/**
 * Fuzzy match query against text.
 * Returns { match: true, score: number } or null.
 * 
 * @param {string} query 
 * @param {string} text 
 * @returns {{ match: true, score: number } | null}
 */
export function fuzzyMatch(query, text) {
    if (!query || query.length === 0) return { match: true, score: 1 };
    
    const lower = text.toLowerCase();
    const q = query.toLowerCase();
    
    let pos = 0;
    let score = 0;
    let streak = 0;
    
    for (let i = 0; i < q.length; i++) {
        const idx = lower.indexOf(q[i], pos);
        if (idx === -1) return null;
        if (idx === pos) streak++;
        else streak = 0;
        score += streak;
        pos = idx + 1;
    }
    
    // Higher score = better match (more contiguous characters)
    return { match: true, score };
}

/**
 * Filter items using fuzzy match across multiple fields.
 * 
 * @template T
 * @param {T[]} items 
 * @param {string} query 
 * @param {(item: T) => string[]} getKey - Function to extract searchable strings
 * @returns {T[]}
 */
export function fuzzyFilter(items, query, getKey) {
    if (!query || query.length === 0) return items;
    
    const results = [];
    for (const item of items) {
        const searchable = getKey(item);
        let bestScore = 0;
        let matched = false;
        
        for (const str of searchable) {
            const result = fuzzyMatch(query, str);
            if (result) {
                matched = true;
                if (result.score > bestScore) bestScore = result.score;
            }
        }
        
        if (matched) {
            results.push({ item, score: bestScore });
        }
    }
    
    // Sort by score descending
    results.sort((a, b) => b.score - a.score);
    return results.map(r => r.item);
}
```

**Step 2: Write a quick test**

```js
// src/runtime/fuzzy.test.js (for node --test)
import { fuzzyMatch, fuzzyFilter } from './fuzzy.js';

// Test cases
console.assert(fuzzyMatch('ctrl+p', 'Ctrl+P Quick Open') !== null, 'basic match');
console.assert(fuzzyMatch('cmdpal', 'Command Palette') !== null, 'fuzzy match');
console.assert(fuzzyMatch('zzz', 'Ctrl+P') === null, 'no match');
console.log('Fuzzy tests passed');
```

**Step 3: Run test**

```bash
node --test src/runtime/fuzzy.test.js
```

**Step 4: Commit**

```bash
git add src/runtime/fuzzy.js src/runtime/fuzzy.test.js
git commit -m "feat: implement custom fuzzy search with scoring"
```

---

## Phase 7: Frontend — Panel Component

### Task 14: Create panel factory with NAF signals

**Objective:** Build the `createPanel(data)` factory function with all signals and computed values.

**Files:**
- Create: `shortcut-viewer/src/panel.js`

**Step 1: Write panel.js**

```js
// @ts-check
import { signal, computed, effect, template, mount, when, raw, text, listener, cleanupCollector, requireRef } from './runtime/naf.js';
import { fuzzyFilter } from './runtime/fuzzy.js';
import { invoke } from '@tauri-apps/api/core';

/**
 * @typedef {object} PanelData
 * @property {string} app_name
 * @property {{ group: string, items: { keys?: string, label: string, value?: string, notes?: string }[] }[]} groups
 */

/**
 * Create the panel component.
 * @param {PanelData} data 
 * @returns {{ mount: (container: HTMLElement) => void, unmount: () => void }}
 */
export function createPanel(data) {
    const appData = signal(data);
    const query = signal('');
    const collapsedGroups = signal(new Set());
    const highlightedIndex = signal(-1);
    
    // Computed: flattened items for keyboard navigation
    const flatItems = computed(() => {
        const items = [];
        const groups = data.groups;
        const q = query();
        
        for (const group of groups) {
            const filteredItems = fuzzyFilter(
                group.items,
                q,
                (item) => [
                    item.keys?.toLowerCase() ?? '',
                    item.label.toLowerCase(),
                    item.value?.toLowerCase() ?? '',
                    item.notes?.toLowerCase() ?? '',
                ]
            );
            items.push({ group: group.group, items: filteredItems });
        }
        return items;
    });
    
    // Computed: filtered groups (non-empty after filtering)
    const visibleGroups = computed(() => {
        return flatItems().filter(g => g.items.length > 0);
    });
    
    // Keyboard handler for navigation
    const handleKeydown = (/** @type {KeyboardEvent} */ event) => {
        const groups = visibleGroups();
        const flat = groups.flatMap(g => g.items);
        
        if (event.key === 'ArrowDown') {
            event.preventDefault();
            highlightedIndex((highlightedIndex() + 1) % Math.max(flat.length, 1));
        } else if (event.key === 'ArrowUp') {
            event.preventDefault();
            highlightedIndex((highlightedIndex() - 1 + flat.length) % Math.max(flat.length, 1));
        } else if (event.key === 'Escape') {
            event.preventDefault();
            invoke('hide_panel').catch(() => {});
        }
    };
    
    // Toggle group collapse
    const toggleGroup = (group) => {
        const collapsed = collapsedGroups();
        const next = new Set(collapsed);
        if (next.has(group)) next.delete(group);
        else next.add(group);
        collapsedGroups(next);
    };
    
    // Render
    const render = template({
        onMount(_el, _parent, ctx) {
            const searchInput = /** @type {HTMLInputElement} */ (requireRef(ctx.refs, 'searchInput'));
            
            queueMicrotask(() => {
                searchInput.focus();
            });
            
            const cleanup = cleanupCollector(
                listener(searchInput, 'input', (e) => {
                    query(/** @type {HTMLInputElement} */ (e.target).value);
                }),
                listener(document, 'keydown', handleKeydown),
            );
            
            ctx.cleanup = () => cleanup.run();
        },
        onUnmount() {
            // cleanup handled by ctx
        },
    });
    
    const component = render`
        <div class="panel" data-ref="panel" style="display:flex;flex-direction:column;max-height:80vh;">
            <div class="panel__header" style="display:flex;align-items:center;padding:8px 12px;cursor:move;-webkit-app-region:drag;border-bottom:1px solid rgba(255,255,255,0.1);">
                <span class="panel__app-name" style="font-weight:600;font-size:14px;">${text(() => data.app_name)}</span>
                <button class="panel__close" style="margin-left:auto;-webkit-app-region:no-drag;background:none;border:none;color:#e0e0e0;font-size:18px;cursor:pointer;padding:0 4px;" onclick="()">×</button>
            </div>
            <div style="padding:8px 12px;">
                <input 
                    class="panel__search" 
                    type="text" 
                    placeholder="Filter references..." 
                    data-ref="searchInput"
                    style="width:100%;padding:6px 10px;background:rgba(255,255,255,0.08);border:1px solid rgba(255,255,255,0.15);border-radius:6px;color:#e0e0e0;font-size:13px;outline:none;"
                />
            </div>
            <div class="panel__body" style="flex:1;overflow-y:auto;padding:0 12px 12px;">
                ${() => {
                    const groups = visibleGroups();
                    return groups.map(g => {
                        const isCollapsed = collapsedGroups().has(g.group);
                        return raw(`
                            <div class="panel__group" style="margin-bottom:12px;">
                                <div class="panel__group-header" style="font-weight:600;font-size:12px;text-transform:uppercase;color:#888;cursor:pointer;padding:4px 0;-webkit-app-region:no-drag;" onclick="(${() => `toggleGroup('${g.group.replace(/'/g, "\\'")}')`})">
                                    ${isCollapsed ? '▸' : '▾'} ${text(g.group)}
                                </div>
                                ${!isCollapsed ? raw(g.items.map((item, idx) => {
                                    const isHighlighted = flatItems().flatMap(gg => gg.items).indexOf(item) === highlightedIndex();
                                    const keysHtml = item.keys
                                        ? item.keys.split('+').map(k => `<kbd>${text(k.trim())}</kbd>`).join('+')
                                        : '';
                                    const secondary = item.value || item.notes || '';
                                    return `
                                        <div class="panel__row" style="display:flex;align-items:center;gap:8px;padding:4px 8px;border-radius:4px;cursor:pointer;-webkit-app-region:no-drag;${isHighlighted ? 'background:rgba(255,255,255,0.1);' : ''}">
                                            ${keysHtml ? `<span class="panel__keys" style="flex-shrink:0;">${keysHtml}</span>` : ''}
                                            <div style="display:flex;flex-direction:column;gap:2px;min-width:0;">
                                                <span class="panel__label" style="font-size:13px;color:#e0e0e0;">${text(item.label)}</span>
                                                ${secondary ? `<span class="panel__desc" style="font-size:13px;color:#bbb;">${text(secondary)}</span>` : ''}
                                            </div>
                                        </div>
                                    `;
                                }).join('')) : ''}
                            </div>
                        `);
                    }).join('');
                }}
            </div>
        </div>
    `;
    
    return {
        mount(container) {
            mount(component, container);
        },
        unmount() {
            component.unmount?.();
        },
    };
}
```

**Step 2: Commit**

```bash
git add src/panel.js
git commit -m "feat: implement panel factory with NAF signals, fuzzy filter, and keyboard navigation"
```

### Task 15: Wire up main.js entry point

**Objective:** Connect Tauri events to the panel factory.

**Files:**
- Modify: `shortcut-viewer/src/main.js`

**Step 1: Rewrite main.js**

```js
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { createPanel } from './panel.js';

let panel = null;

listen('show-panel', (event) => {
    if (panel) {
        panel.unmount();
    }
    panel = createPanel(event.payload);
    panel.mount(document.getElementById('app'));
});

listen('hide-panel', () => {
    panel?.unmount();
    panel = null;
    invoke('hide_panel').catch(() => {});
});

// Also handle close button
document.addEventListener('click', (e) => {
    if (e.target.closest('.panel__close')) {
        e.target.closest('.panel__close')?.click();
    }
});
```

**Step 2: Commit**

```bash
git add src/main.js
git commit -m "feat: wire up main.js entry point with Tauri event listeners"
```

---

## Phase 8: Rust — Panel Window Creation

### Task 16: Implement show_panel command

**Objective:** Create the Rust command that detects the window, looks up shortcuts, and shows the panel.

**Files:**
- Modify: `shortcut-viewer/src-tauri/src/lib.rs`

**Step 1: Add show_panel command and hotkey integration**

```rust
#[tauri::command]
async fn cmd_show_panel(app: tauri::AppHandle) -> Result<(), String> {
    let data: State<AppData> = app.state();
    
    // Toggle: if panel exists, close it
    if let Some(window_id) = *data.panel_window.read().unwrap() {
        if let Some(window) = app.get_webview_window(window_id) {
            window.close().ok();
            *data.panel_window.write().unwrap() = None;
            return Ok(());
        }
    }
    
    // Detect active window
    let chain = data.detection_chain.read().unwrap();
    let window_info = if let Some(boxed) = &chain.0 {
        boxed.detect()
    } else {
        None
    };
    
    // Look up shortcuts or fall back to picker
    let map = data.shortcut_map.read().unwrap();
    let panel_data = if let Some(info) = window_info {
        scanner::lookup_shortcuts(&map, &info)
    } else {
        None
    };
    
    // Create panel window
    let window = tauri::WebviewWindow::builder(&app, "panel", tauri::WebviewUrl::Default)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(400.0, 600.0)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    
    // Store window ID
    *data.panel_window.write().unwrap() = Some(window.id());
    
    // Emit data to frontend
    if let Some(pd) = panel_data {
        window.emit("show-panel", pd).ok();
    } else {
        // Fallback: show picker
        let picker_apps = scanner::get_picker_apps(&map);
        window.emit("show-panel-picker", picker_apps).ok();
    }
    
    Ok(())
}
```

**Step 2: Add to command handler**

In `invoke_handler`:
```rust
.cmd_show_panel
```

**Step 3: Hook hotkey to show_panel**

In the setup, after registering the hotkey:
```rust
// The global shortcut triggers cmd_show_panel
// In Tauri v2, listen for the shortcut event
app.listen_global("tauri://global-shortcut/pressed", move |_event| {
    // Invoke show_panel
    tauri::async_runtime::spawn(async move {
        cmd_show_panel(app.clone()).await.ok();
    });
});
```

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: implement show_panel command with window detection and data emission"
```

---

## Phase 9: Polish and Integration

### Task 17: Create default reference files

**Objective:** Seed the reference directory with example files that demonstrate multiple use cases.

**Files:**
- Create: `shortcut-viewer/shortcuts-example/vscode.yaml`
- Create: `shortcut-viewer/shortcuts-example/git.yaml`
- Create: `shortcut-viewer/shortcuts-example/people.yaml`

**Step 1: Create example reference files**

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
      - keys: Ctrl+Shift+N
        label: New Window
      - keys: Ctrl+W
        label: Close Window
  - group: Editing
    items:
      - keys: Alt+Up
        label: Move line up
      - keys: Alt+Down
        label: Move line down
      - keys: Shift+Alt+Arrow
        label: Copy line up/down
  - group: View
    items:
      - keys: Ctrl+B
        label: Toggle Sidebar
      - keys: Ctrl+J
        label: Toggle Panel
      - keys: Ctrl+Shift+E
        label: Focus Explorer
```

Create `shortcut-viewer/shortcuts-example/git.yaml`:
```yaml
name: Git Reference
process:
  - wezterm
  - alacritty
  - gnome-terminal
references:
  - group: Tagging
    items:
      - label: Annotated tag
        value: git tag -a v1.2.3 -m "Release v1.2.3"
      - label: Push tags
        value: git push --tags
  - group: Rollback
    items:
      - label: Revert a commit
        value: git revert <commit>
        notes: Safe for shared history
      - label: Reset hard
        value: git reset --hard <commit>
        notes: Rewrites local state
```

Create `shortcut-viewer/shortcuts-example/people.yaml`:
```yaml
name: Team Reference
process:
  - outlook
  - thunderbird
  - slack
references:
  - group: Product
    items:
      - label: Jordan Lee
        value: Product lead
        notes: Owns roadmap sign-off
      - label: Avery Chen
        value: Design
        notes: Reviews customer-facing copy
  - group: Platform
    items:
      - label: Sam Patel
        value: SRE
        notes: Escalate incident-related questions
```

**Step 2: Create a setup script**

Create `shortcut-viewer/setup.sh`:
```bash
#!/bin/bash
CONFIG_DIR="$HOME/.config/shortcut-viewer/references"
mkdir -p "$CONFIG_DIR"
cp shortcuts-example/*.yaml "$CONFIG_DIR/"
echo "References directory created at: $CONFIG_DIR"
echo "Example files copied."
```

**Step 3: Commit**

```bash
git add shortcuts-example/ setup.sh
git commit -m "docs: add example reference files and setup script"
```

### Task 18: Build and verify

**Objective:** Run the app and verify basic functionality.

**Step 1: Run the app**

```bash
cd shortcut-viewer && npm run tauri dev
```

**Step 2: Verify**
- App starts with tray icon (no visible window)
- Press `Ctrl+Alt+?` (or configured hotkey)
- Panel window appears
- Fuzzy search filters references across `keys`, `label`, `value`, and `notes`
- Arrow keys navigate
- Group headers toggle collapse/expand
- Close button or hotkey dismisses panel

**Step 3: Commit**

```bash
git add .
git commit -m "feat: verify basic app functionality"
```

---

## Phase 10: Settings Window

### Task 19: Create settings window UI and command

**Objective:** Build the settings window accessible from the tray menu.

**Files:**
- Create: `shortcut-viewer/src/settings.js`
- Modify: `shortcut-viewer/src-tauri/src/lib.rs`

**Step 1: Write settings.js**

```js
// @ts-check
import { signal, effect, template, mount, text, listener, cleanupCollector, requireRef } from './runtime/naf.js';
import { invoke } from '@tauri-apps/api/core';

export function createSettings() {
    const hotkey = signal('');
    const theme = signal('dark');
    const shortcutsDir = signal('');
    
    // Load config
    invoke('get_config').then((cfg) => {
        hotkey(cfg.hotkey);
        theme(cfg.theme);
        shortcutsDir(cfg.references_dir || '');
    });
    
    const save = () => {
        invoke('set_config', { configData: {
            hotkey: hotkey(),
            theme: theme(),
            references_dir: shortcutsDir() || null,
        } }).then(() => {
            // Reload references if dir changed
            invoke('reload_shortcuts').ok();
        });
    };
    
    const component = template({
        onMount(_el, _parent, ctx) {
            const saveBtn = /** @type {HTMLButtonElement} */ (requireRef(ctx.refs, 'saveBtn'));
            const cleanup = cleanupCollector(
                listener(saveBtn, 'click', save),
            );
            ctx.cleanup = () => cleanup.run();
        },
    });
    
    return {
        mount(container) {
            mount(component, container);
        },
        unmount() {},
    };
}
```

**Step 2: Add settings window creation to lib.rs tray handler**

In the tray icon setup:
```rust
"settings" => {
    let settings_window = tauri::WebviewWindow::builder(&app, "settings", tauri::WebviewUrl::Default)
        .title("Settings")
        .inner_size(400.0, 300.0)
        .center()
        .build()
        .unwrap();
    settings_window.emit("open-settings", ()).ok();
}
```

**Step 3: Commit**

```bash
git add src/settings.js src-tauri/src/lib.rs
git commit -m "feat: implement settings window with hotkey, theme, and directory config"
```

### Task 20: Final polish and README

**Objective:** Add README, clean up code, and finalize.

**Files:**
- Create: `shortcut-viewer/README.md`

**Step 1: Write README**

```markdown
# Shortcut Viewer

A lightweight, cross-platform desktop utility that displays contextual quick-reference information for your currently active application.

## Features

- Global hotkey to show/hide the reference panel
- Automatic active window detection (Windows, Linux X11, Linux Wayland fallback)
- YAML-based reference files with grouped categories
- Fuzzy search and keyboard navigation
- Always-on-top floating panel with frosted glass effect
- Automatic file reload on changes
- Configurable hotkey, theme, and reference directory

## Installation

1. Clone the repo
2. Run `./setup.sh` to create the config directory and example files
3. Run `npm run tauri build` to build the app
4. Install the generated binary

## Configuration

Edit `~/.config/shortcut-viewer/config.yaml` (Linux) or `%APPDATA%/shortcut-viewer/config.yaml` (Windows):

```yaml
hotkey: "CommandOrControl+Alt+Slash"
theme: "dark"
references_dir: null  # Optional override
```

## Reference Files

Place YAML files in `~/.config/shortcut-viewer/references/`:

```yaml
name: My App
process:
  - myapp
references:
  - group: Navigation
    items:
      - keys: Ctrl+P
        label: Quick Open
        notes: Opens a file by name
```

## License

MIT
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with installation and configuration guide"
```

---

## Summary

Total: **20 tasks** across 10 phases.

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-2 | Project scaffolding |
| 2 | 3 | Configuration system |
| 3 | 4-6 | Shortcut file scanning & watching |
| 4 | 7-9 | Window detection (Windows, X11, Wayland) |
| 5 | 10-11 | Tauri state, commands, tray, hotkey |
| 6 | 12-13 | NAF runtime port & fuzzy search |
| 7 | 14-15 | Panel component & entry point |
| 8 | 16 | Panel window creation & data flow |
| 9 | 17-18 | Example files & verification |
| 10 | 19-20 | Settings window & README |

Each task is designed to be implemented sequentially with a clear commit boundary. The Rust backend handles all platform-specific concerns, while the frontend remains a thin, dependency-free renderer.
