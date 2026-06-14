use crate::config;
use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, RwLock};
use zbus::blocking::{connection::Builder as ConnectionBuilder, Connection, Proxy};

const DBUS_SERVICE_NAME: &str = "com.sellsword.JustPeek";
const DBUS_OBJECT_PATH: &str = "/ActiveWindow";
const KWIN_SCRIPTING_SERVICE: &str = "org.kde.KWin";
const KWIN_SCRIPTING_PATH: &str = "/Scripting";
const KWIN_SCRIPTING_INTERFACE: &str = "org.kde.kwin.Scripting";
const KWIN_SCRIPT_PLUGIN_NAME: &str = "justpeek-active-window";

static ACTIVE_WINDOW: OnceLock<RwLock<Option<WindowInfo>>> = OnceLock::new();
static BRIDGE: OnceLock<Mutex<Option<KdeWaylandBridge>>> = OnceLock::new();

pub struct KdeWaylandDetector;

pub struct KdeWaylandBridge {
    _connection: Connection,
    _script_id: i32,
    _script_path: PathBuf,
}

impl WindowDetector for KdeWaylandDetector {
    fn name(&self) -> &'static str {
        "wayland-kde"
    }

    fn is_available(&self) -> bool {
        std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "wayland")
            && std::env::var("XDG_CURRENT_DESKTOP")
                .map(|value| value.to_lowercase().contains("kde"))
                .unwrap_or(false)
    }

    fn detect(&self) -> Option<WindowInfo> {
        active_window_cache().read().unwrap().clone()
    }
}

pub fn install_bridge() -> Result<(), String> {
    if !KdeWaylandDetector.is_available() {
        return Ok(());
    }

    let bridge = KdeWaylandBridge::install()?;
    *bridge_slot().lock().unwrap() = Some(bridge);
    Ok(())
}

fn active_window_cache() -> &'static RwLock<Option<WindowInfo>> {
    ACTIVE_WINDOW.get_or_init(|| RwLock::new(None))
}

fn bridge_slot() -> &'static Mutex<Option<KdeWaylandBridge>> {
    BRIDGE.get_or_init(|| Mutex::new(None))
}

struct ActiveWindowReceiver;

#[zbus::interface(name = "com.sellsword.JustPeek.ActiveWindow")]
impl ActiveWindowReceiver {
    fn update_active_window(
        &self,
        window_title: &str,
        resource_class: &str,
        resource_name: &str,
        desktop_file: &str,
    ) {
        let candidates = [resource_class, resource_name, desktop_file]
            .into_iter()
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>();

        let next = if candidates.is_empty() && window_title.trim().is_empty() {
            None
        } else {
            let process_name = candidates.first().copied().unwrap_or("unknown").to_string();

            Some(WindowInfo::from_candidates(
                process_name,
                window_title.to_string(),
                candidates,
            ))
        };

        // eprintln!(
        //     "[justpeek] kde bridge update: title='{}' class='{}' name='{}' desktop='{}'",
        //     window_title, resource_class, resource_name, desktop_file
        // );
        *active_window_cache().write().unwrap() = next;
    }
}

impl KdeWaylandBridge {
    fn install() -> Result<Self, String> {
        let script_path = ensure_script_file()?;

        let connection = ConnectionBuilder::session()
            .map_err(|err| err.to_string())?
            .name(DBUS_SERVICE_NAME)
            .map_err(|err| err.to_string())?
            .serve_at(DBUS_OBJECT_PATH, ActiveWindowReceiver)
            .map_err(|err| err.to_string())?
            .build()
            .map_err(|err| err.to_string())?;

        let scripting = Proxy::new(
            &connection,
            KWIN_SCRIPTING_SERVICE,
            KWIN_SCRIPTING_PATH,
            KWIN_SCRIPTING_INTERFACE,
        )
        .map_err(|err| err.to_string())?;

        let _ignored: Result<bool, _> = scripting.call("unloadScript", &(KWIN_SCRIPT_PLUGIN_NAME));
        let script_path_str = script_path.to_string_lossy().into_owned();
        let script_id: i32 = scripting
            .call(
                "loadScript",
                &(script_path_str.as_str(), KWIN_SCRIPT_PLUGIN_NAME),
            )
            .map_err(|err| err.to_string())?;

        let script_proxy = Proxy::new(
            &connection,
            KWIN_SCRIPTING_SERVICE,
            format!("/Scripting/Script{script_id}"),
            "org.kde.kwin.Script",
        )
        .map_err(|err| err.to_string())?;

        let _: () = script_proxy
            .call("run", &())
            .map_err(|err| err.to_string())?;

        Ok(Self {
            _connection: connection,
            _script_id: script_id,
            _script_path: script_path,
        })
    }
}

fn ensure_script_file() -> Result<PathBuf, String> {
    let script_dir = config::config_dir().join("runtime");
    std::fs::create_dir_all(&script_dir).map_err(|err| err.to_string())?;

    let script_path = script_dir.join("kwin-active-window.js");
    std::fs::write(&script_path, kwin_script_source()).map_err(|err| err.to_string())?;
    Ok(script_path)
}

fn kwin_script_source() -> &'static str {
    r#"
const DBUS_SERVICE_NAME = "com.sellsword.JustPeek";
const DBUS_OBJECT_PATH = "/ActiveWindow";
const DBUS_INTERFACE = "com.sellsword.JustPeek.ActiveWindow";

let previousWindow = null;
let previousCaptionListener = null;
let previousClassListener = null;
let previousNameListener = null;

function isJustPeekWindow(window) {
    const resourceClass = (window.resourceClass || "").toLowerCase();
    const resourceName = (window.resourceName || "").toLowerCase();
    return resourceClass.indexOf("justpeek") !== -1 || resourceName.indexOf("justpeek") !== -1;
}

function emitWindow(window) {
    if (!window || !window.normalWindow || isJustPeekWindow(window)) {
        return;
    }

    callDBus(
        DBUS_SERVICE_NAME,
        DBUS_OBJECT_PATH,
        DBUS_INTERFACE,
        "UpdateActiveWindow",
        window.caption || "",
        window.resourceClass || "",
        window.resourceName || "",
        window.desktopFileName || ""
    );
}

function disconnectPreviousWindow() {
    if (!previousWindow) {
        return;
    }

    try {
        if (previousCaptionListener) {
            previousWindow.captionChanged.disconnect(previousCaptionListener);
        }
        if (previousClassListener) {
            previousWindow.resourceClassChanged.disconnect(previousClassListener);
        }
        if (previousNameListener) {
            previousWindow.resourceNameChanged.disconnect(previousNameListener);
        }
    } catch (error) {
        // ignore stale window disconnects
    }
}

function subscribeToWindow(window) {
    if (!window || previousWindow === window) {
        return;
    }

    disconnectPreviousWindow();

    previousCaptionListener = function() { emitWindow(window); };
    previousClassListener = function() { emitWindow(window); };
    previousNameListener = function() { emitWindow(window); };

    try {
        window.captionChanged.connect(previousCaptionListener);
        window.resourceClassChanged.connect(previousClassListener);
        window.resourceNameChanged.connect(previousNameListener);
    } catch (error) {
        // ignore missing signals
    }

    previousWindow = window;
}

function updateActiveWindow(window) {
    if (!window || !window.normalWindow || isJustPeekWindow(window)) {
        return;
    }

    emitWindow(window);
    subscribeToWindow(window);
}

workspace.windowActivated.connect(updateActiveWindow);

if (workspace.activeWindow) {
    updateActiveWindow(workspace.activeWindow);
}
"#
}
