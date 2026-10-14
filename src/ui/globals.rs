//! Globals for the UI
//!
//! This module contains all the Ratatui-based UI components for ramwise.

pub const RSS_NAME: &str = if cfg!(target_os = "linux") {
    "RSS"
} else if cfg!(target_os = "windows") {
    "Real"
} else {
    "r"
};
pub const VSS_NAME: &str = if cfg!(target_os = "linux") {
    "VSS"
} else if cfg!(target_os = "windows") {
    "Virt"
} else {
    "v"
};

pub const SIGKILL_NAME: &str = if cfg!(target_os = "linux") {
    "SIGKILL"
} else if cfg!(target_os = "windows") {
    "End task"
} else {
    "Kill forcefully"
};

pub const SIGTERM_NAME: &str = if cfg!(target_os = "linux") {
    "SIGTERM"
} else if cfg!(target_os = "windows") {
    "Close gracefully"
} else {
    "Ask to exit"
};
