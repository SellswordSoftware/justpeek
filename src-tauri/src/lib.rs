mod config;
mod detection;
mod hotkey;
mod scanner;
mod watcher;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, RwLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::webview::WebviewWindowBuilder;
use tauri::{
    AppHandle, Emitter, Listener, Manager, PhysicalPosition, State, WebviewUrl, WebviewWindow,
    WindowEvent,
};

const PANEL_WINDOW_LABEL: &str = "panel";
const PANEL_WINDOW_WIDTH: f64 = 420.0;
const PANEL_WINDOW_HEIGHT: f64 = 640.0;

#[derive(Clone)]
enum PanelDisplayPayload {
    References(scanner::PanelData),
    Picker(Vec<scanner::PickerApp>),
}

pub struct AppData {
    config: RwLock<config::Config>,
    shortcut_map: RwLock<HashMap<String, scanner::ShortcutFile>>,
    picker_apps: RwLock<Vec<scanner::PickerApp>>,
    picker_panels: RwLock<HashMap<String, scanner::PanelData>>,
    panel_window: RwLock<Option<String>>,
    panel_visible: RwLock<bool>,
    panel_ready: RwLock<bool>,
    pending_panel_payload: RwLock<Option<PanelDisplayPayload>>,
    panel_position: Mutex<Option<PhysicalPosition<i32>>>,
    detection_chain: RwLock<detection::DetectionChain>,
    watcher: Mutex<Option<watcher::ShortcutWatcher>>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RuntimeInfo {
    os: String,
    session_type: Option<String>,
    hotkey_editable: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let cfg = config::read_config();
            let references_dir = config::references_dir_path(&cfg);
            std::fs::create_dir_all(&references_dir)
                .expect("Failed to create references directory");

            #[cfg(target_os = "linux")]
            if let Err(error) = detection::wayland_kde::install_bridge() {
                debug_log(format!(
                    "kde wayland active-window bridge failed to install: {error}"
                ));
            }

            let mut chain = detection::DetectionChain::new();
            chain.init();

            let scan_result = scanner::scan_shortcuts(&references_dir);
            let app_data = AppData {
                config: RwLock::new(cfg.clone()),
                shortcut_map: RwLock::new(scan_result.shortcut_map),
                picker_apps: RwLock::new(scan_result.picker_apps),
                picker_panels: RwLock::new(scan_result.picker_panels),
                panel_window: RwLock::new(None),
                panel_visible: RwLock::new(false),
                panel_ready: RwLock::new(false),
                pending_panel_payload: RwLock::new(None),
                panel_position: Mutex::new(None),
                detection_chain: RwLock::new(chain),
                watcher: Mutex::new(None),
            };
            app.manage(app_data);

            install_reference_watcher(app.handle(), references_dir)?;
            install_panel_ready_handler(app.handle());
            create_panel_window(app.handle())?;
            hotkey::install_hotkey_listener(app.handle(), &cfg.hotkey);
            install_hotkey_toggle_handler(app.handle());
            build_tray(app)?;
            if let Some(window) = app.get_webview_window("main") {
                debug_log("setup: configuring main/settings window behavior");
                configure_settings_window_behavior(&window);
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_show_panel,
            cmd_hide_panel,
            cmd_get_config,
            cmd_get_runtime_info,
            cmd_set_config,
            cmd_reload_shortcuts,
            cmd_open_settings_window,
            cmd_open_shortcuts_dir,
            cmd_open_external_url,
            cmd_log_client_event,
            cmd_get_picker_apps,
            cmd_load_picker_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running JustPeek");
}

fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let reload_item = MenuItem::with_id(app, "reload", "Reload References", true, None::<&str>)?;
    let open_dir_item = MenuItem::with_id(
        app,
        "open_dir",
        "Open References Directory",
        true,
        None::<&str>,
    )?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let tray_menu = Menu::with_items(
        app,
        &[&reload_item, &open_dir_item, &settings_item, &quit_item],
    )?;

    let default_icon = app.default_window_icon().cloned();
    let app_handle = app.handle().clone();

    let mut tray = TrayIconBuilder::new()
        .menu(&tray_menu)
        .tooltip("JustPeek")
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => {
                debug_log("tray action: quit");
                app.exit(0)
            }
            "reload" => {
                debug_log("tray action: reload references");
                let _ = reload_shortcuts(app);
            }
            "open_dir" => {
                debug_log("tray action: open references directory");
                let _ = open_shortcuts_dir();
            }
            "settings" => {
                debug_log("tray action: open settings");
                let _ = show_settings_window(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, _event| {
            debug_log("tray icon event: show panel");
            let _ = show_panel_window(&tray.app_handle());
        });

    if let Some(icon) = default_icon {
        tray = tray.icon(icon);
    }

    let _tray = tray.build(&app_handle)?;
    Ok(())
}

fn install_hotkey_toggle_handler(app: &AppHandle) {
    let app_handle = app.clone();

    app.listen("justpeek://toggle-panel", move |_event| {
        debug_log("received toggle-panel event");
        let _ = show_panel_window(&app_handle);
    });

    // Task 11 infrastructure: a future global shortcut registration should emit
    // `justpeek://toggle-panel`, which keeps the toggle behavior centralized.
}

#[cfg(debug_assertions)]
fn debug_log(message: impl AsRef<str>) {
    append_log_line(message.as_ref());
    eprintln!("[justpeek] {}", message.as_ref());
}

#[cfg(not(debug_assertions))]
fn debug_log(message: impl AsRef<str>) {
    append_log_line(message.as_ref());
}

fn append_log_line(message: &str) {
    let dir = config::config_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(config::log_path())
    {
        Ok(file) => file,
        Err(_) => return,
    };

    let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0,
    };

    let _ = writeln!(file, "[{timestamp}] {message}");
}

fn install_panel_ready_handler(app: &AppHandle) {
    let app_handle = app.clone();

    app.listen("justpeek://panel-ready", move |event| {
        debug_log(format!("received panel-ready event payload={}", event.payload()));
        if !event.payload().contains(PANEL_WINDOW_LABEL) {
            return;
        }

        let state = app_handle.state::<AppData>();
        *state.panel_ready.write().unwrap() = true;

        let pending = state.pending_panel_payload.write().unwrap().take();
        if let Some(payload) = pending {
            let _ = show_panel_payload(&app_handle, payload);
        }
    });
}

fn remember_panel_position(app: &AppHandle, position: PhysicalPosition<i32>) {
    let state = app.state::<AppData>();
    *state.panel_position.lock().unwrap() = Some(position);
}

fn set_panel_visible(app: &AppHandle, visible: bool) {
    let state = app.state::<AppData>();
    *state.panel_visible.write().unwrap() = visible;
}

fn is_panel_visible(app: &AppHandle) -> bool {
    let state = app.state::<AppData>();
    let visible = *state.panel_visible.read().unwrap();
    visible
}

fn snapshot_panel_position(window: &WebviewWindow) {
    match window.outer_position() {
        Ok(position) => remember_panel_position(&window.app_handle(), position),
        Err(error) => {
            debug_log(format!("panel position snapshot unavailable: {error}"));
        }
    }
}

fn restore_panel_position(window: &WebviewWindow) {
    let saved_position = {
        let state = window.app_handle().state::<AppData>();
        let position = *state.panel_position.lock().unwrap();
        position
    };

    let Some(position) = saved_position else {
        return;
    };

    if let Err(error) = window.set_position(position) {
        debug_log(format!("panel position restore unavailable: {error}"));
    }
}

fn install_reference_watcher(app: &tauri::AppHandle, dir: std::path::PathBuf) -> tauri::Result<()> {
    let handle = app.clone();
    let shortcut_watcher = watcher::ShortcutWatcher::new(dir, move |path| {
        let _ = reload_shortcuts_for_path(&handle, &path);
    })
    .map_err(|err| tauri::Error::from(std::io::Error::other(err.to_string())))?;

    let state = app.state::<AppData>();
    *state.watcher.lock().unwrap() = Some(shortcut_watcher);
    Ok(())
}

fn reload_shortcuts_for_path(app: &tauri::AppHandle, path: &std::path::Path) -> Result<(), String> {
    let state = app.state::<AppData>();
    let next = scanner::scan_shortcuts(path);
    *state.shortcut_map.write().unwrap() = next.shortcut_map;
    *state.picker_apps.write().unwrap() = next.picker_apps;
    *state.picker_panels.write().unwrap() = next.picker_panels;
    Ok(())
}

fn create_panel_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(PANEL_WINDOW_LABEL).is_some() {
        return Ok(());
    }

    debug_log("creating panel window");
    let panel_window = WebviewWindowBuilder::new(
        app,
        PANEL_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("JustPeek")
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .visible_on_all_workspaces(true)
    .skip_taskbar(true)
    .resizable(true)
    .focused(false)
    .visible(false)
    .min_inner_size(400., 300.)
    .inner_size(PANEL_WINDOW_WIDTH, PANEL_WINDOW_HEIGHT)
    // .center()
    .build()
    .map_err(|err| err.to_string())?;

    debug_log("panel window created successfully");

    let app_handle = app.clone();
    let panel_window_handle = panel_window.clone();
    panel_window.on_window_event(move |event| match event {
        WindowEvent::Moved(position) => {
            debug_log(format!("panel window moved to x={}, y={}", position.x, position.y));
            remember_panel_position(&app_handle, *position);
        }
        WindowEvent::CloseRequested { api, .. } => {
            debug_log("panel window close requested; hiding instead");
            api.prevent_close();
            snapshot_panel_position(&panel_window_handle);
            set_panel_visible(&app_handle, false);
            let _ = panel_window_handle.hide();
        }
        WindowEvent::Destroyed => {
            debug_log("panel window destroyed");
            let state = app_handle.state::<AppData>();
            *state.panel_window.write().unwrap() = None;
            *state.panel_visible.write().unwrap() = false;
            *state.panel_ready.write().unwrap() = false;
        }
        _ => {}
    });

    let state = app.state::<AppData>();
    *state.panel_window.write().unwrap() = Some(PANEL_WINDOW_LABEL.to_string());
    Ok(())
}

fn hide_panel_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(PANEL_WINDOW_LABEL) else {
        return Ok(());
    };

    snapshot_panel_position(&window);
    set_panel_visible(app, false);
    window.hide().map_err(|err| err.to_string())
}

fn show_panel_payload(app: &AppHandle, payload: PanelDisplayPayload) -> Result<(), String> {
    let Some(window) = app.get_webview_window(PANEL_WINDOW_LABEL) else {
        return Err("Panel window not found".to_string());
    };

    debug_log("show_panel_payload: dispatching payload to panel window");
    match payload {
        PanelDisplayPayload::References(panel_data) => {
            debug_log(format!(
                "show_panel_payload: references app_name={}",
                panel_data.app_name
            ));
            window
                .emit("show-panel", panel_data)
                .map_err(|err| err.to_string())?;
        }
        PanelDisplayPayload::Picker(picker_apps) => {
            debug_log(format!(
                "show_panel_payload: picker app count={}",
                picker_apps.len()
            ));
            window
                .emit("show-panel-picker", picker_apps)
                .map_err(|err| err.to_string())?;
        }
    }

    debug_log("show_panel_payload: calling window.show()");
    window.show().map_err(|err| err.to_string())?;
    restore_panel_position(&window);
    set_panel_visible(app, true);
    debug_log("show_panel_payload: calling window.set_focus()");
    window.set_focus().map_err(|err| err.to_string())
}

fn show_panel_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(PANEL_WINDOW_LABEL) else {
        return Err("Panel window not found".to_string());
    };

    if is_panel_visible(app) {
        debug_log("panel already visible; hiding current panel window");
        return hide_panel_window(app);
    }

    let state = app.state::<AppData>();

    let detection_trace = {
        let chain = state.detection_chain.read().unwrap();
        chain.detect_with_trace()
    };
    for line in &detection_trace.log_lines {
        debug_log(line);
    }

    let shortcut_map = state.shortcut_map.read().unwrap().clone();
    let payload = if let Some(window) = detection_trace.window.as_ref() {
        let lookup_trace = scanner::lookup_shortcuts_with_trace(&shortcut_map, window);
        for line in &lookup_trace.log_lines {
            debug_log(line);
        }

        if let Some(panel_data) = lookup_trace.panel_data {
            debug_log(format!(
                "panel payload: showing references for '{}'",
                panel_data.app_name
            ));
            PanelDisplayPayload::References(panel_data)
        } else {
            let picker_apps = state.picker_apps.read().unwrap().clone();
            debug_log(format!(
                "panel payload: no contextual match; showing picker with {} app(s)",
                picker_apps.len()
            ));
            PanelDisplayPayload::Picker(picker_apps)
        }
    } else {
        let picker_apps = state.picker_apps.read().unwrap().clone();
        debug_log(format!(
            "panel payload: no active window detected; showing picker with {} app(s)",
            picker_apps.len()
        ));
        PanelDisplayPayload::Picker(picker_apps)
    };

    if *state.panel_ready.read().unwrap() {
        debug_log("show_panel_window: panel already ready");
        show_panel_payload(app, payload)
    } else {
        debug_log("panel window not ready yet; queued payload until ready event arrives");
        *state.pending_panel_payload.write().unwrap() = Some(payload);
        debug_log("show_panel_window: calling window.show() while waiting for ready");
        window.show().map_err(|err| err.to_string())?;
        restore_panel_position(&window);
        set_panel_visible(app, true);
        debug_log("show_panel_window: calling window.set_focus() while waiting for ready");
        window.set_focus().map_err(|err| err.to_string())?;
        Ok(())
    }
}

fn configure_settings_window_behavior(window: &WebviewWindow) {
    let window_handle = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            debug_log("settings window close requested; hiding instead");
            api.prevent_close();
            let _ = window_handle.hide();
        }
    });
}

fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("Settings window not found".to_string());
    };

    debug_log("show_settings_window: calling window.show()");
    window.show().map_err(|err| err.to_string())?;
    debug_log("show_settings_window: calling window.set_focus()");
    window.set_focus().map_err(|err| err.to_string())?;
    debug_log("show_settings_window: emitting open-settings");
    window
        .emit("open-settings", ())
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn reload_shortcuts(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppData>();
    let cfg = state.config.read().unwrap().clone();
    let path = config::references_dir_path(&cfg);
    reload_shortcuts_for_path(app, &path)
}

fn open_shortcuts_dir() -> Result<(), String> {
    let cfg = config::read_config();
    let dir = config::references_dir_path(&cfg);
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&dir)
        .spawn()
        .map_err(|err| err.to_string())?;
    #[cfg(windows)]
    std::process::Command::new("explorer")
        .arg(&dir)
        .spawn()
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn open_external_url(url: &str) -> Result<(), String> {
    let normalized = url.trim();
    if !(normalized.starts_with("https://") || normalized.starts_with("http://")) {
        return Err("Only http:// and https:// URLs are supported".to_string());
    }

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(normalized)
        .spawn()
        .map_err(|err| err.to_string())?;
    #[cfg(windows)]
    std::process::Command::new("explorer")
        .arg(normalized)
        .spawn()
        .map_err(|err| err.to_string())?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(normalized)
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok(())
}

fn current_session_type() -> Option<String> {
    std::env::var("XDG_SESSION_TYPE")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

fn runtime_info() -> RuntimeInfo {
    let session_type = current_session_type();
    let hotkey_editable = cfg!(target_os = "linux") && session_type.as_deref() != Some("wayland");

    RuntimeInfo {
        os: std::env::consts::OS.to_string(),
        session_type,
        hotkey_editable,
    }
}

#[tauri::command]
async fn cmd_show_panel(app: tauri::AppHandle) -> Result<(), String> {
    show_panel_window(&app)
}

#[tauri::command]
async fn cmd_hide_panel(_app: tauri::AppHandle, window: WebviewWindow) -> Result<(), String> {
    snapshot_panel_position(&window);
    set_panel_visible(&window.app_handle(), false);
    window.hide().map_err(|err| err.to_string())
}

#[tauri::command]
async fn cmd_get_config(app: tauri::AppHandle) -> Result<config::Config, String> {
    let state: State<'_, AppData> = app.state();
    let config = state.config.read().unwrap().clone();
    Ok(config)
}

#[tauri::command]
async fn cmd_get_runtime_info() -> Result<RuntimeInfo, String> {
    Ok(runtime_info())
}

#[tauri::command]
async fn cmd_set_config(app: tauri::AppHandle, config_data: config::Config) -> Result<(), String> {
    hotkey::validate_hotkey(&config_data.hotkey)?;
    if !config::is_supported_preferred_shortcut_os(&config_data.preferred_shortcut_os) {
        return Err(format!(
            "Unsupported preferred shortcut OS '{}'. Use one of: auto, macos, windows, linux.",
            config_data.preferred_shortcut_os
        ));
    }
    if !config::is_supported_shortcut_display_mode(&config_data.shortcut_display_mode) {
        return Err(format!(
            "Unsupported shortcut display mode '{}'. Use one of: current, all.",
            config_data.shortcut_display_mode
        ));
    }

    let state: State<'_, AppData> = app.state();
    let old_config = state.config.read().unwrap().clone();
    *state.config.write().unwrap() = config_data.clone();
    config::save_config(&config_data);

    let old_dir = config::references_dir_path(&old_config);
    let new_dir = config::references_dir_path(&config_data);
    if old_dir != new_dir {
        std::fs::create_dir_all(&new_dir).map_err(|err| err.to_string())?;
        reload_shortcuts_for_path(&app, &new_dir)?;
        install_reference_watcher(&app, new_dir).map_err(|err| err.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn cmd_reload_shortcuts(app: tauri::AppHandle) -> Result<(), String> {
    reload_shortcuts(&app)
}

#[tauri::command]
async fn cmd_open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    show_settings_window(&app)
}

#[tauri::command]
async fn cmd_open_shortcuts_dir() -> Result<(), String> {
    open_shortcuts_dir()
}

#[tauri::command]
async fn cmd_open_external_url(url: String) -> Result<(), String> {
    open_external_url(&url)
}

#[tauri::command]
async fn cmd_log_client_event(message: String) -> Result<(), String> {
    debug_log(format!("client: {message}"));
    Ok(())
}

#[tauri::command]
async fn cmd_get_picker_apps(app: tauri::AppHandle) -> Result<Vec<scanner::PickerApp>, String> {
    let state: State<'_, AppData> = app.state();
    let picker_apps = state.picker_apps.read().unwrap().clone();
    Ok(picker_apps)
}

#[tauri::command]
async fn cmd_load_picker_app(
    app: tauri::AppHandle,
    picker_id: String,
) -> Result<scanner::PanelData, String> {
    let state: State<'_, AppData> = app.state();
    let panel = state
        .picker_panels
        .read()
        .unwrap()
        .get(&picker_id)
        .cloned()
        .ok_or_else(|| format!("Reference file not found for picker id '{picker_id}'"));
    panel
}
