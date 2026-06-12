use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;
use std::path::Path;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

pub struct WindowsDetector;

impl WindowDetector for WindowsDetector {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn detect(&self) -> Option<WindowInfo> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            return None;
        }

        let title_len = unsafe { GetWindowTextLengthW(hwnd) };
        let mut title_buffer = vec![0u16; title_len as usize + 1];
        let copied = unsafe { GetWindowTextW(hwnd, &mut title_buffer) };
        let window_title = String::from_utf16_lossy(&title_buffer[..copied as usize]);

        let mut pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }

        let process_name = get_process_name(pid)?;

        Some(WindowInfo::from_process_name(process_name, window_title))
    }
}

fn get_process_name(pid: u32) -> Option<String> {
    let process = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
    }
    .ok()?;

    let process_name = query_process_name(process);
    unsafe {
        let _ = CloseHandle(process);
    }

    process_name
}

fn query_process_name(process: HANDLE) -> Option<String> {
    let mut buffer = vec![0u16; 1024];
    let mut size = buffer.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(process, 0, PWSTR(buffer.as_mut_ptr()), &mut size).ok()?;
    }

    let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
    let file_name = Path::new(&full_path).file_name()?.to_string_lossy();
    Some(file_name.to_string())
}
