mod config;
mod detection;
mod hotkey;
mod scanner;
mod watcher;

use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Listener, Manager, State, WebviewUrl, WebviewWindow, WindowEvent};

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
    panel_window: RwLock<Option<String>>,
    panel_ready: RwLock<bool>,
    pending_panel_payload: RwLock<Option<PanelDisplayPayload>>,
    detection_chain: RwLock<detection::DetectionChain>,
    watcher: Mutex<Option<watcher::ShortcutWatcher>>,
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

            let reference_map = scanner::scan_shortcuts(&references_dir);
            let app_data = AppData {
                config: RwLock::new(cfg.clone()),
                shortcut_map: RwLock::new(reference_map),
                panel_window: RwLock::new(None),
                panel_ready: RwLock::new(false),
                pending_panel_payload: RwLock::new(None),
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
                configure_settings_window_behavior(&window);
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_show_panel,
            cmd_hide_panel,
            cmd_get_config,
            cmd_set_config,
            cmd_reload_shortcuts,
            cmd_open_shortcuts_dir,
            cmd_get_picker_apps,
            cmd_load_picker_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running JustPeek");
}

fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let reload_item = MenuItem::with_id(app, "reload", "Reload References", true, None::<&str>)?;
    let open_dir_item =
        MenuItem::with_id(app, "open_dir", "Open References Directory", true, None::<&str>)?;
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
            "quit" => app.exit(0),
            "reload" => {
                let _ = reload_shortcuts(app);
            }
            "open_dir" => {
                let _ = open_shortcuts_dir();
            }
            "settings" => {
                let _ = show_settings_window(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, _event| {
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
        let _ = show_panel_window(&app_handle);
    });

    // Task 11 infrastructure: a future global shortcut registration should emit
    // `justpeek://toggle-panel`, which keeps the toggle behavior centralized.
}

fn debug_log(message: impl AsRef<str>) {
    //eprintln!("[justpeek] {}", message.as_ref());
}

fn install_panel_ready_handler(app: &AppHandle) {
    let app_handle = app.clone();

    app.listen("justpeek://panel-ready", move |event| {
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

fn reload_shortcuts_for_path(
    app: &tauri::AppHandle,
    path: &std::path::Path,
) -> Result<(), String> {
    let state = app.state::<AppData>();
    let next = scanner::scan_shortcuts(path);
    *state.shortcut_map.write().unwrap() = next;
    Ok(())
}

fn create_panel_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(PANEL_WINDOW_LABEL).is_some() {
        return Ok(());
    }

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
    .inner_size(PANEL_WINDOW_WIDTH, PANEL_WINDOW_HEIGHT)
    // .center()
    .build()
    .map_err(|err| err.to_string())?;

    let app_handle = app.clone();
    let panel_window_handle = panel_window.clone();
    panel_window.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = panel_window_handle.hide();
        }
        WindowEvent::Destroyed => {
            let state = app_handle.state::<AppData>();
            *state.panel_window.write().unwrap() = None;
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

    window.hide().map_err(|err| err.to_string())
}

fn show_panel_payload(app: &AppHandle, payload: PanelDisplayPayload) -> Result<(), String> {
    let Some(window) = app.get_webview_window(PANEL_WINDOW_LABEL) else {
        return Err("Panel window not found".to_string());
    };

    match payload {
        PanelDisplayPayload::References(panel_data) => {
            window.emit("show-panel", panel_data).map_err(|err| err.to_string())?;
        }
        PanelDisplayPayload::Picker(picker_apps) => {
            window
                .emit("show-panel-picker", picker_apps)
                .map_err(|err| err.to_string())?;
        }
    }

    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())
}

fn show_panel_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(PANEL_WINDOW_LABEL) else {
        return Err("Panel window not found".to_string());
    };

    if window.is_visible().map_err(|err| err.to_string())? {
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
            let picker_apps = scanner::get_picker_apps(&shortcut_map);
            debug_log(format!(
                "panel payload: no contextual match; showing picker with {} app(s)",
                picker_apps.len()
            ));
            PanelDisplayPayload::Picker(picker_apps)
        }
    } else {
        let picker_apps = scanner::get_picker_apps(&shortcut_map);
        debug_log(format!(
            "panel payload: no active window detected; showing picker with {} app(s)",
            picker_apps.len()
        ));
        PanelDisplayPayload::Picker(picker_apps)
    };

    if *state.panel_ready.read().unwrap() {
        show_panel_payload(app, payload)
    } else {
        debug_log("panel window not ready yet; queued payload until ready event arrives");
        *state.pending_panel_payload.write().unwrap() = Some(payload);
        Ok(())
    }
}

fn configure_settings_window_behavior(window: &WebviewWindow) {
    let window_handle = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window_handle.hide();
        }
    });
}

fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("Settings window not found".to_string());
    };

    window
        .emit("open-settings", ())
        .map_err(|err| err.to_string())?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
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

#[tauri::command]
async fn cmd_show_panel(app: tauri::AppHandle) -> Result<(), String> {
    show_panel_window(&app)
}

#[tauri::command]
async fn cmd_hide_panel(_app: tauri::AppHandle, window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|err| err.to_string())
}

#[tauri::command]
async fn cmd_get_config(app: tauri::AppHandle) -> Result<config::Config, String> {
    let state: State<'_, AppData> = app.state();
    let config = state.config.read().unwrap().clone();
    Ok(config)
}

#[tauri::command]
async fn cmd_set_config(
    app: tauri::AppHandle,
    config_data: config::Config,
) -> Result<(), String> {
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
async fn cmd_open_shortcuts_dir() -> Result<(), String> {
    open_shortcuts_dir()
}

#[tauri::command]
async fn cmd_get_picker_apps(app: tauri::AppHandle) -> Result<Vec<scanner::PickerApp>, String> {
    let state: State<'_, AppData> = app.state();
    let shortcut_map = state.shortcut_map.read().unwrap().clone();
    Ok(scanner::get_picker_apps(&shortcut_map))
}

#[tauri::command]
async fn cmd_load_picker_app(
    app: tauri::AppHandle,
    picker_id: String,
) -> Result<scanner::PanelData, String> {
    let state: State<'_, AppData> = app.state();
    let shortcut_map = state.shortcut_map.read().unwrap().clone();
    scanner::lookup_picker_app(&shortcut_map, &picker_id)
        .ok_or_else(|| format!("Reference file not found for picker id '{picker_id}'"))
}
