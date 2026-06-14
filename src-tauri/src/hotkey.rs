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
        let preferred_trigger = match portal_hotkey_string(&hotkey) {
            Ok(trigger) => trigger,
            Err(error) => {
                eprintln!("JustPeek hotkey is invalid: {error}");
                return;
            }
        };

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

        let shortcut = NewShortcut::new(
            TOGGLE_SHORTCUT_ID,
            "Toggle the JustPeek contextual reference panel",
        )
        .preferred_trigger(Some(preferred_trigger.as_str()));

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

pub fn validate_hotkey(hotkey: &str) -> Result<(), String> {
    parse_hotkey_trigger(hotkey).map(|_| ())
}

fn parse_hotkey_trigger(hotkey: &str) -> Result<String, String> {
    let trimmed = hotkey.trim();
    if trimmed.is_empty() {
        return Err("Hotkey cannot be empty.".to_string());
    }

    let parts = trimmed.split('+').map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err("Hotkey contains an empty segment. Use syntax like Ctrl+Shift+K.".to_string());
    }

    let mut trigger = String::new();
    let mut key: Option<String> = None;

    for part in parts {
        let lower = part.to_ascii_lowercase();
        match lower.as_str() {
            "commandorcontrol" | "control" | "ctrl" => trigger.push_str("<Ctrl>"),
            "alt" | "option" => trigger.push_str("<Alt>"),
            "shift" => trigger.push_str("<Shift>"),
            "meta" | "super" | "command" | "cmd" => trigger.push_str("<Super>"),
            _ => {
                if key.is_some() {
                    return Err(
                        "Hotkey must have exactly one non-modifier key at the end.".to_string()
                    );
                }
                key = Some(lower);
            }
        }
    }

    let key = key.ok_or_else(|| "Hotkey must include a final key.".to_string())?;
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
        _ => {
            return Err(format!(
                "Unsupported hotkey key '{key}'. Use a single letter, digit, or a supported named key such as Slash, Enter, Escape, Tab, or an arrow key."
            ))
        }
    });

    Ok(trigger)
}

#[cfg(target_os = "linux")]
fn portal_hotkey_string(hotkey: &str) -> Result<String, String> {
    parse_hotkey_trigger(hotkey)
}
