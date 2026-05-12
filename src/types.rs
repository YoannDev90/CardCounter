use gpui::*;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq)]
pub enum AppMode {
    Manual,
    Interactive,
}

pub struct ScanThrottle {
    pub last_scan: Instant,
    pub last_code: String,
}

impl ScanThrottle {
    pub fn new() -> Self {
        Self {
            last_scan: Instant::now(),
            last_code: String::new(),
        }
    }
}
