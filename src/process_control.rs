//! Process signal helpers for stop/kill actions.

use crate::ui::globals::{SIGKILL_NAME, SIGTERM_NAME};
use std::io;

/// User action mapped to a Unix signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAction {
    Terminate,
    Kill,
}

impl SignalAction {
    pub fn as_label(&self) -> &'static str {
        match self {
            SignalAction::Terminate => SIGTERM_NAME,
            SignalAction::Kill => SIGKILL_NAME,
        }
    }
}

/// Outcome of sending a signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalResult {
    Sent,
    NotFound,
    PermissionDenied,
    InvalidTarget,
    Failed(String),
}

/// Send a signal to a process with safety checks.
pub fn send_signal(pid: i32, action: SignalAction) -> SignalResult {
    if pid <= 1 {
        return SignalResult::InvalidTarget;
    }

    #[cfg(target_os = "linux")]
    let signal = match action {
        SignalAction::Terminate => SIGTERM,
        SignalAction::Kill => SIGKILL,
    };
    // TODO Finish windows kill
    #[cfg(target_os = "windows")]
    let signal = match action {
        // both of these need handles
        SignalAction::Terminate => a, // DestroyWindow and something for console
        SignalAction::Kill => a,      //TerminateProcess
    };

    map_signal_result(send_raw_signal(pid, signal))
}

fn map_signal_result(result: io::Result<()>) -> SignalResult {
    match result {
        Ok(()) => SignalResult::Sent,
        Err(err) if err.raw_os_error() == Some(ESRCH) => SignalResult::NotFound,
        Err(err) if err.raw_os_error() == Some(EPERM) => SignalResult::PermissionDenied,
        Err(err) => SignalResult::Failed(err.to_string()),
    }
}
#[cfg(target_os = "linux")]
const SIGTERM: i32 = 15;
#[cfg(target_os = "linux")]
const SIGKILL: i32 = 9;
const EPERM: i32 = 1;
const ESRCH: i32 = 3;
#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}
#[cfg(target_os = "linux")]
fn send_raw_signal(pid: i32, sig: i32) -> io::Result<()> {
    // SAFETY: libc kill is called with plain integer pid/signal values on Linux.
    let rc = unsafe { kill(pid, sig) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_signal_rejects_invalid_pid() {
        assert_eq!(
            send_signal(1, SignalAction::Terminate),
            SignalResult::InvalidTarget
        );
        assert_eq!(
            send_signal(0, SignalAction::Kill),
            SignalResult::InvalidTarget
        );
        assert_eq!(
            send_signal(-2, SignalAction::Terminate),
            SignalResult::InvalidTarget
        );
    }

    #[test]
    fn map_signal_result_maps_common_errors() {
        assert_eq!(
            map_signal_result(Err(io::Error::from_raw_os_error(ESRCH))),
            SignalResult::NotFound
        );
        assert_eq!(
            map_signal_result(Err(io::Error::from_raw_os_error(EPERM))),
            SignalResult::PermissionDenied
        );
    }

    #[test]
    fn signal_labels_are_stable() {
        assert_eq!(SignalAction::Terminate.as_label(), "SIGTERM");
        assert_eq!(SignalAction::Kill.as_label(), "SIGKILL");
    }
}
