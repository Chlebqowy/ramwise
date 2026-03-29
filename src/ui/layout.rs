//! Layout management for the UI

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// Main layout manager
pub struct Layout {
    /// Header height
    pub header_height: u16,
    /// Main panel height
    pub main_height: u16,
    /// Bottom panel height
    pub bottom_height: u16,
    /// Left panel width percentage
    pub left_width_percent: u16,
    /// Percentage of the top panel for the right side split (by default contains details and graph)
    pub side_vertical_split_percent: u16,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            header_height: 1,
            main_height: 0, // Will be remaining space unless constraint types are changed
            bottom_height: 4,
            left_width_percent: 40,
            side_vertical_split_percent: 60,
        }
    }

    /// Calculate all layout areas from the terminal size
    pub fn calculate(&self, area: Rect) -> LayoutAreas {
        // Split into header, main, and bottom
        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                /// Constraints for header, main, and bottom panels. You can use either Length or Min for either. Length will give a fixed size, while Min will take up remaining space after Length constraints are applied. If there are multiple Min constraints, the space will be split between them as close to equal as possible while respecting the minimum sizes.
                Constraint::Length(self.header_height),
                Constraint::Min(self.main_height),
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
                /// You can make this use lenght and min as well and you can do it the same way as the vertical split. The variables are not created though. Here's an example of how you could do it:
                /// Constraint::Length(10),
                /// Constraint::Min(5),
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
                /// Same as for horizontal 
                Constraint::Percentage(self.side_vertical_split_percent),
                Constraint::Percentage(100 - self.side_vertical_split_percent),
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
