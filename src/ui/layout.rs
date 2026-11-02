//! Layout management for the UI

use crate::ui::Theme;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect, Spacing};
/// Main layout manager
#[derive(Debug, Clone)]
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
    /// If true, neighbouring panels overlap by one cell so they share a border
    pub merge_borders: bool,
}

impl Layout {
    pub fn new(theme: &Theme) -> Self {
        Self {
            header_height: theme.header_height,
            center_height: theme.center_height,
            bottom_height: theme.bottom_height,
            left_width_percent: theme.left_width_percent.min(100),
            side_vertical_split_percent: theme.side_vertical_split_percent.min(100),

            invert_horizontal_split: theme.invert_horizontal_split,
            invert_side_vertical_split: theme.invert_side_vertical_split,
            put_insights_on_top: theme.put_insights_on_top,
            merge_borders: theme.merge_borders,
        }
    }

    /// Calculate all layout areas from the terminal size
    pub fn calculate(&self, area: Rect) -> LayoutAreas {
        // The header has no border, so it never overlaps the panels
        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(self.header_height), Constraint::Fill(1)])
            .split(area);
        let (header, body) = (vertical[0], vertical[1]);

        // (Process list | details and graph) above or below insights
        let (main, bottom) = self.split_insights(body);
        let (left_panel, side) = self.split_columns(main);
        let (detail_panel, graph_panel) = self.split_side(side);

        LayoutAreas {
            header,
            left_panel,
            detail_panel,
            graph_panel,
            bottom,
        }
    }

    /// Space between panels: with merged borders, neighbours share a border cell
    fn spacing(&self) -> Spacing {
        if self.merge_borders {
            Spacing::Overlap(1)
        } else {
            Spacing::Space(0)
        }
    }

    /// Split off the insights panel: returns (rest, insights)
    /// The insights panel always keeps its own border (never shared), so it
    /// stays a separate rounded box
    fn split_insights(&self, area: Rect) -> (Rect, Rect) {
        let insights = Constraint::Length(self.bottom_height);
        let rest = Constraint::Min(self.center_height);
        let split = |constraints: [Constraint; 2]| {
            RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area)
        };
        if self.put_insights_on_top {
            let parts = split([insights, rest]);
            (parts[1], parts[0])
        } else {
            let parts = split([rest, insights]);
            (parts[0], parts[1])
        }
    }

    /// Split into the process list and the side column: returns (list, side)
    fn split_columns(&self, area: Rect) -> (Rect, Rect) {
        // Fill weights instead of percentages so overlapping borders still
        // divide the space as configured
        let parts = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .spacing(self.spacing())
            .constraints([
                Constraint::Fill(self.left_width_percent),
                Constraint::Fill(100 - self.left_width_percent),
            ])
            .split(area);
        if self.invert_horizontal_split {
            (parts[1], parts[0])
        } else {
            (parts[0], parts[1])
        }
    }

    /// Split the side column into details and graph: returns (detail, graph)
    fn split_side(&self, area: Rect) -> (Rect, Rect) {
        let parts = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .spacing(self.spacing())
            .constraints([
                Constraint::Fill(self.side_vertical_split_percent),
                Constraint::Fill(100 - self.side_vertical_split_percent),
            ])
            .split(area);
        if self.invert_side_vertical_split {
            (parts[1], parts[0])
        } else {
            (parts[0], parts[1])
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new(&Theme::dark())
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

    fn layout(configure: impl FnOnce(&mut Layout)) -> LayoutAreas {
        let mut layout = Layout::new(&Theme::dark());
        configure(&mut layout);
        layout.calculate(Rect::new(0, 0, 100, 50))
    }

    #[test]
    fn default_layout() {
        let areas = layout(|_| {});
        assert_eq!(areas.header, Rect::new(0, 0, 100, 1));
        // Full-width insights at the bottom
        assert_eq!(areas.bottom.height, 4);
        assert_eq!(areas.bottom.bottom(), 50);
        assert_eq!(areas.bottom.width, 100);
        // Process list gets the larger share of the width (60%)
        assert!(areas.left_panel.width > areas.detail_panel.width);
        assert!(areas.left_panel.x < areas.detail_panel.x);
    }

    #[test]
    fn merged_borders_share_one_cell_between_neighbours() {
        let areas = layout(|_| {});
        // Process list and side column share the vertical border
        assert_eq!(areas.left_panel.right() - 1, areas.detail_panel.x);
        // Detail and graph share a horizontal border
        assert_eq!(areas.detail_panel.bottom() - 1, areas.graph_panel.y);
        // Insights keep their own border, right below the panels
        assert_eq!(areas.graph_panel.bottom(), areas.bottom.y);
        assert_eq!(areas.left_panel.bottom(), areas.bottom.y);
    }

    #[test]
    fn separate_borders_do_not_overlap() {
        let areas = layout(|l| l.merge_borders = false);
        assert_eq!(areas.left_panel.right(), areas.detail_panel.x);
        assert_eq!(areas.detail_panel.bottom(), areas.graph_panel.y);
        assert_eq!(areas.graph_panel.bottom(), areas.bottom.y);
    }

    #[test]
    fn insights_never_overlap_other_panels_content() {
        for merge in [true, false] {
            for invert in [true, false] {
                for top in [true, false] {
                    let areas = layout(|l| {
                        l.merge_borders = merge;
                        l.invert_horizontal_split = invert;
                        l.put_insights_on_top = top;
                    });
                    for panel in [areas.left_panel, areas.detail_panel, areas.graph_panel] {
                        // Not even a shared border cell
                        assert!(
                            !areas.bottom.intersects(panel),
                            "merge={merge} invert={invert} top={top}"
                        );
                    }
                    assert!(!areas.bottom.intersects(areas.header));
                }
            }
        }
    }

    #[test]
    fn separate_borders_give_insights_their_own_rows() {
        let areas = layout(|l| l.merge_borders = false);
        assert_eq!(areas.bottom, Rect::new(0, 46, 100, 4));
        assert_eq!(areas.left_panel.y, 1);
        assert_eq!(areas.left_panel.bottom(), 46);
    }

    #[test]
    fn put_insights_on_top_inverts_vertical_order() {
        let areas = layout(|l| {
            l.put_insights_on_top = true;
            l.merge_borders = false;
        });
        // Insights (bottom area) should be directly below the header
        assert_eq!(areas.bottom.y, 1);
        assert_eq!(areas.bottom.height, 4);
        // Left panel should start below insights
        assert_eq!(areas.left_panel.y, 1 + 4);
    }

    #[test]
    fn invert_horizontal_and_side_split() {
        let areas = layout(|l| {
            l.invert_horizontal_split = true;
            l.invert_side_vertical_split = true;
        });
        // Side column should now be on the left (x = 0)
        assert_eq!(areas.detail_panel.x, 0);
        assert_eq!(areas.graph_panel.x, 0);
        // Graph panel should be above detail panel
        assert!(areas.graph_panel.y < areas.detail_panel.y);
    }

    #[test]
    fn side_split_uses_its_own_percentage() {
        let areas = layout(|l| {
            l.merge_borders = false;
            l.side_vertical_split_percent = 75;
        });
        let total = f64::from(areas.detail_panel.height + areas.graph_panel.height);
        let expected = total * 0.75;
        assert!((f64::from(areas.detail_panel.height) - expected).abs() <= 1.0);
    }
}
