use tauri::{AppHandle, Emitter};

#[cfg(target_os = "linux")]
use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
#[cfg(target_os = "linux")]
use futures_util::StreamExt;

const TOGGLE_SHORTCUT_ID: &str = "toggle-panel";

pub fn install_hotkey_listener(app: &AppHandle, hotkey: &str) {
    #[cfg(target_os = "linux")]
    {
        install_linux_hotkey_listener(app.clone(), hotkey.to_string());
    }
}

#[cfg(target_os = "linux")]
fn install_linux_hotkey_listener(app: AppHandle, hotkey: String) {
    tauri::async_runtime::spawn(async move {
        let portal = match GlobalShortcuts::new().await {
            Ok(portal) => portal,
            Err(error) => {
                eprintln!("JustPeek failed to connect to GlobalShortcuts portal: {error}");
                return;
            }
        };

        let session = match portal.create_session().await {
            Ok(session) => session,
            Err(error) => {
                eprintln!("JustPeek failed to create hotkey session: {error}");
                return;
            }
        };

        let preferred_trigger = portal_hotkey_string(&hotkey);
        let shortcut = NewShortcut::new(
            TOGGLE_SHORTCUT_ID,
            "Toggle the JustPeek contextual reference panel",
        )
        .preferred_trigger(preferred_trigger.as_deref());

        match portal.bind_shortcuts(&session, &[shortcut], None).await {
            Ok(request) => {
                if let Err(error) = request.response() {
                    eprintln!("JustPeek hotkey registration was rejected: {error}");
                    return;
                }
            }
            Err(error) => {
                eprintln!("JustPeek failed to request hotkey registration: {error}");
                return;
            }
        }

        let mut activations = match portal.receive_activated().await {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("JustPeek failed to subscribe to hotkey activations: {error}");
                return;
            }
        };

        while let Some(event) = activations.next().await {
            if event.shortcut_id() == TOGGLE_SHORTCUT_ID {
                let _ = app.emit("justpeek://toggle-panel", ());
            }
        }
    });
}

#[cfg(target_os = "linux")]
fn portal_hotkey_string(hotkey: &str) -> Option<String> {
    let mut trigger = String::new();
    let mut key: Option<String> = None;

    for part in hotkey.split('+').map(str::trim).filter(|part| !part.is_empty()) {
        let lower = part.to_ascii_lowercase();
        match lower.as_str() {
            "commandorcontrol" | "control" | "ctrl" => trigger.push_str("<Ctrl>"),
            "alt" | "option" => trigger.push_str("<Alt>"),
            "shift" => trigger.push_str("<Shift>"),
            "meta" | "super" | "command" | "cmd" => trigger.push_str("<Super>"),
            _ => key = Some(lower),
        }
    }

    let key = key?;
    trigger.push_str(match key.as_str() {
        "slash" => "slash",
        "question" => "question",
        "space" => "space",
        "enter" | "return" => "Return",
        "escape" | "esc" => "Escape",
        "tab" => "Tab",
        "up" | "arrowup" => "Up",
        "down" | "arrowdown" => "Down",
        "left" | "arrowleft" => "Left",
        "right" | "arrowright" => "Right",
        single if single.len() == 1 => single,
        _ => return None,
    });

    Some(trigger)
}
