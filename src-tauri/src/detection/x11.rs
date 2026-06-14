use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;
use std::path::Path;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt as _, Window};

pub struct X11Detector;

impl WindowDetector for X11Detector {
    fn name(&self) -> &'static str {
        "x11"
    }

    fn is_available(&self) -> bool {
        std::env::var("DISPLAY").is_ok()
    }

    fn detect(&self) -> Option<WindowInfo> {
        let (conn, screen_num) = x11rb::connect(None).ok()?;
        let screen = conn.setup().roots.get(screen_num)?;
        let active_window_atom = intern_atom(&conn, b"_NET_ACTIVE_WINDOW")?;
        let active_window = get_window_property(&conn, screen.root, active_window_atom)?;

        let window_title = get_window_title(&conn, active_window).unwrap_or_default();
        let process_name = get_window_pid(&conn, active_window)
            .and_then(get_process_name)
            .unwrap_or_else(|| "unknown".to_string());

        Some(WindowInfo::from_process_name(process_name, window_title))
    }
}

fn intern_atom<C: Connection>(conn: &C, name: &[u8]) -> Option<Atom> {
    conn.intern_atom(false, name)
        .ok()?
        .reply()
        .ok()
        .map(|reply| reply.atom)
}

fn get_window_property<C: Connection>(conn: &C, root: Window, atom: Atom) -> Option<Window> {
    conn.get_property(false, root, atom, AtomEnum::WINDOW, 0, 1)
        .ok()?
        .reply()
        .ok()?
        .value32()?
        .next()
}

fn get_window_title<C: Connection>(conn: &C, window: Window) -> Option<String> {
    let utf8_string = intern_atom(conn, b"UTF8_STRING")?;
    let net_wm_name = intern_atom(conn, b"_NET_WM_NAME")?;

    let utf8_title = conn
        .get_property(false, window, net_wm_name, utf8_string, 0, 1024)
        .ok()?
        .reply()
        .ok()?;

    if !utf8_title.value.is_empty() {
        return Some(String::from_utf8_lossy(&utf8_title.value).to_string());
    }

    let wm_name = conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)
        .ok()?
        .reply()
        .ok()?;

    if wm_name.value.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&wm_name.value).to_string())
    }
}

fn get_window_pid<C: Connection>(conn: &C, window: Window) -> Option<u32> {
    let pid_atom = intern_atom(conn, b"_NET_WM_PID")?;
    conn.get_property(false, window, pid_atom, AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?
        .value32()?
        .next()
}

fn get_process_name(pid: u32) -> Option<String> {
    let comm_path = format!("/proc/{pid}/comm");
    if let Ok(name) = std::fs::read_to_string(&comm_path) {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let cmdline_path = format!("/proc/{pid}/cmdline");
    let cmdline = std::fs::read(&cmdline_path).ok()?;
    let first = cmdline.split(|byte| *byte == 0).next()?;
    let raw = String::from_utf8_lossy(first);
    let file_name = Path::new(raw.as_ref()).file_name()?.to_string_lossy();
    Some(file_name.to_string())
}
