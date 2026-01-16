//! Layout management for the UI

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// Main layout manager
pub struct Layout {
    /// Header height
    pub header_height: u16,
    /// Bottom panel height
    pub bottom_height: u16,
    /// Left panel width percentage
    pub left_width_percent: u16,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            header_height: 1,
            bottom_height: 4,
            left_width_percent: 40,
        }
    }

    /// Calculate all layout areas from the terminal size
    pub fn calculate(&self, area: Rect) -> LayoutAreas {
        // Split into header, main, and bottom
        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.header_height),
                Constraint::Min(10),
                Constraint::Length(self.bottom_height),
            ])
            .split(area);

        let header = vertical[0];
        let main = vertical[1];
        let bottom = vertical[2];

        // Split main into left and right panels
        let horizontal = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_width_percent),
                Constraint::Percentage(100 - self.left_width_percent),
            ])
            .split(main);

        let left_panel = horizontal[0];
        let right_panel = horizontal[1];

        // Split right panel into detail and graph
        let right_split = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(right_panel);

        let detail_panel = right_split[0];
        let graph_panel = right_split[1];

        LayoutAreas {
            header,
            left_panel,
            detail_panel,
            graph_panel,
            bottom,
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

/// Computed layout areas
#[derive(Debug, Clone, Copy)]
pub struct LayoutAreas {
    /// Top header bar
    pub header: Rect,
    /// Left panel (process list)
    pub left_panel: Rect,
    /// Right top (detail view)
    pub detail_panel: Rect,
    /// Right bottom (graph)
    pub graph_panel: Rect,
    /// Bottom panel (insights)
    pub bottom: Rect,
}
