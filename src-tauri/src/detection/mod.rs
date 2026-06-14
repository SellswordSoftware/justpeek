use crate::scanner::WindowInfo;

#[cfg(target_os = "linux")]
pub mod wayland_kde;
#[cfg(target_os = "linux")]
pub mod wayland_sway;
#[cfg(windows)]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod x11;

pub trait WindowDetector: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn detect(&self) -> Option<WindowInfo>;
}

pub struct DetectionChain {
    detectors: Vec<Box<dyn WindowDetector>>,
}

pub struct DetectionTrace {
    pub window: Option<WindowInfo>,
    pub log_lines: Vec<String>,
}

impl DetectionChain {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        let mut detectors: Vec<Box<dyn WindowDetector>> = Vec::new();

        #[cfg(windows)]
        {
            detectors.push(Box::new(windows::WindowsDetector));
        }

        #[cfg(target_os = "linux")]
        {
            if is_x11() {
                detectors.push(Box::new(x11::X11Detector));
            }

            if is_kde_wayland() {
                detectors.push(Box::new(wayland_kde::KdeWaylandDetector));
            }

            if is_sway() {
                detectors.push(Box::new(wayland_sway::SwayDetector));
            }
        }

        self.detectors = detectors;
    }

    pub fn detect_with_trace(&self) -> DetectionTrace {
        let mut log_lines = Vec::new();

        if self.detectors.is_empty() {
            #[cfg(target_os = "linux")]
            {
                let session_type =
                    std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "<unset>".to_string());
                let desktop =
                    std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "<unset>".to_string());
                log_lines.push(format!(
                    "window detection: no detectors initialized for session_type='{session_type}' desktop='{desktop}'"
                ));
            }

            #[cfg(not(target_os = "linux"))]
            log_lines.push("window detection: no detectors initialized".to_string());

            return DetectionTrace {
                window: None,
                log_lines,
            };
        }

        for detector in &self.detectors {
            let detector_name = detector.name();
            let available = detector.is_available();
            log_lines.push(format!(
                "window detection: detector={detector_name} available={available}"
            ));

            if !available {
                continue;
            }

            if let Some(info) = detector.detect() {
                log_lines.push(format!(
                    "window detection: detector={detector_name} selected process='{}' title='{}'",
                    info.process_name, info.window_title
                ));
                return DetectionTrace {
                    window: Some(info),
                    log_lines,
                };
            }

            log_lines.push(format!(
                "window detection: detector={detector_name} returned no active window"
            ));
        }

        log_lines.push("window detection: no detector produced an active window".to_string());
        DetectionTrace {
            window: None,
            log_lines,
        }
    }
}

#[cfg(target_os = "linux")]
fn is_x11() -> bool {
    std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "x11")
}

#[cfg(target_os = "linux")]
fn is_sway() -> bool {
    std::env::var("SWAYSOCK").is_ok()
}

#[cfg(target_os = "linux")]
fn is_kde_wayland() -> bool {
    let is_wayland = std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "wayland");
    let is_kde = std::env::var("XDG_CURRENT_DESKTOP")
        .map(|value| value.to_lowercase().contains("kde"))
        .unwrap_or(false);

    is_wayland && is_kde
}
