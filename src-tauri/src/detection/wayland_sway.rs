use crate::detection::WindowDetector;
use crate::scanner::WindowInfo;

pub struct SwayDetector;

impl WindowDetector for SwayDetector {
    fn name(&self) -> &'static str {
        "wayland-sway"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn detect(&self) -> Option<WindowInfo> {
        None
    }
}
