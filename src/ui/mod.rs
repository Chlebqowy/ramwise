//! Terminal UI components
//!
//! This module contains all the Ratatui-based UI components for ramwise.

pub mod layout;
pub mod theme;
pub mod widgets;
pub mod globals;

pub use globals::{RSS_NAME, VSS_NAME};
pub use layout::Layout;
pub use theme::Theme;
