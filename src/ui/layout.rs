//! Layout management for the UI

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};
use ratatui::layout::Spacing;

/// Main layout manager
pub struct Layout {
    /// Header height
    pub header_height: u16,
    /// Main panel height
    pub center_height: u16,
    /// Bottom panel height
    pub bottom_height: u16,
    /// Left panel width percentage
    pub left_width_percent: u16,
    /// Percentage of the top panel for the right side split (by default contains details and graph)
    pub side_vertical_split_percent: u16,
    /// If this value is true, the left panel will be on the right and the right panel will be on the left.
    pub invert_horizontal_split: bool,
    /// If this value is true, the top panel will be on the bottom and the bottom panel will be on the top.
    pub invert_side_vertical_split: bool,
    /// If this value is true, the insights panel will be below the header and above main
    pub put_insights_on_top: bool,
    /// If true, 2 borders will become 1. So for example, if without it a slice of the layout is content||content, with it it would be content|content
    pub collapse_borders: bool
}

impl Layout {
    pub fn new() -> Self {
        Self {
            header_height: 1,
            center_height: 10,
            bottom_height: 4,
            left_width_percent: 40,
            side_vertical_split_percent: 60,

            invert_horizontal_split: false,
            invert_side_vertical_split: false,
            put_insights_on_top: false,

            collapse_borders: true,
        }
    }

    /// Calculate all layout areas from the terminal size
    pub fn calculate(&self, area: Rect) -> LayoutAreas {
        let overlap: u16 = if self.collapse_borders {1} else {0}; // In the function because it needs self
        // Split into header, main, and bottom (insights)
        let (header, main, bottom) = if self.put_insights_on_top {
            let vertical = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(self.header_height),
                    Constraint::Length(self.bottom_height),
                    Constraint::Min(self.center_height),
                ])
                .split(area);
            (vertical[0], vertical[2], vertical[1])
        } else {
            let vertical = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(self.header_height),
                    Constraint::Min(self.center_height),
                    Constraint::Length(self.bottom_height),
                ])
                .split(area);
            (vertical[0], vertical[1], vertical[2])
        };

        // Split main into left and right panels
        let horizontal = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_width_percent),
                Constraint::Percentage(100 - self.left_width_percent),
            ])
            .split(main);

        let (left_panel, right_panel) = if self.invert_horizontal_split {
            (horizontal[1], horizontal[0])
        } else {
            (horizontal[0], horizontal[1])
        };

        // Split right panel into detail and graph
        let right_split = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .spacing(Spacing::Overlap(overlap))
            .constraints([
                Constraint::Percentage(self.side_vertical_split_percent),
                Constraint::Percentage(100 - self.side_vertical_split_percent),
            ])
            .split(right_panel);

        let (detail_panel, graph_panel) = if self.invert_side_vertical_split {
            (right_split[1], right_split[0])
        } else {
            (right_split[0], right_split[1])
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_calculation() {
        let layout = Layout::new();
        let area = Rect::new(0, 0, 100, 50);
        let areas = layout.calculate(area);

        assert_eq!(areas.header.y, 0);
        assert_eq!(areas.header.height, 1);
        assert_eq!(areas.bottom.height, 4);
        assert_eq!(areas.bottom.y, 50 - 4);
        assert!(
            areas.left_panel.width < areas.detail_panel.width
                || areas.left_panel.x < areas.detail_panel.x
        );
    }

    #[test]
    fn put_insights_on_top_inverts_vertical_order() {
        let mut layout = Layout::new();
        layout.put_insights_on_top = true;
        let area = Rect::new(0, 0, 100, 50);
        let areas = layout.calculate(area);

        assert_eq!(areas.header.y, 0);
        assert_eq!(areas.header.height, 1);
        // Insights (bottom area) should be directly below the header
        assert_eq!(areas.bottom.y, 1);
        assert_eq!(areas.bottom.height, 4);
        // Left panel should start below insights
        assert_eq!(areas.left_panel.y, 1 + 4);
    }

    #[test]
    fn invert_horizontal_and_side_split() {
        let mut layout = Layout::new();
        layout.invert_horizontal_split = true;
        layout.invert_side_vertical_split = true;
        let area = Rect::new(0, 0, 100, 50);
        let areas = layout.calculate(area);

        // Right panel should now be on the left (x = 0)
        assert_eq!(areas.detail_panel.x, 0);
        assert_eq!(areas.graph_panel.x, 0);
        // Graph panel should be above detail panel
        assert!(areas.graph_panel.y < areas.detail_panel.y);
    }
}
