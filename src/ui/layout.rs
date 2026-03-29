//! Layout management for the UI

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

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
}

impl Layout {
    pub fn new() -> Self {
        Self {
            header_height: 1,
            center_height: 5, // Will be remaining space unless constraint types are changed
            bottom_height: 0,
            left_width_percent: 40,
            side_vertical_split_percent: 60,
             
            invert_horizontal_split: false,
            invert_side_vertical_split: false,
            /* IF YOU WANT TO CHANGE THIS, YOU ALSO SHOULD CHANGE center_height and bottom_height. The main part becomes bottom, the right part becomes center.
            If you want to make it similar to the default, you can do 
            Constraint::Length(self.header_height),
            Constraint::Lenght(self.bottom_height),
            Constraint::Min(self.center_height)
            
            in 
                let vertical = RatatuiLayout::default() 
                    .direction(Direction::Vertical) 
                    .constraints([

            This will feel scuffed if you want to change some stuff outside this section though.*/

            put_insights_on_top: true, 
        }
    }

    /// Calculate all layout areas from the terminal size
    pub fn calculate(&self, area: Rect) -> LayoutAreas {
        // Some issues with doc comments here that I can't fix. Look at compile warnings for info.
        /** Split into header, main, and bottom
        Constraints for header, main, and bottom panels. You can use either Length or Min for either. Length will give a fixed size, while Min will take up remaining space after Length constraints are applied. If there are multiple Min constraints, the space will be split between them as close to equal as possible while respecting the minimum sizes.
        This will still apply based on direction, even if panels are inverted! **/
        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                // TODO Make these depend on layout. Right now it's top center bottom, should be top main bottom. Always, even if the panels are inverted. So default inverted layout is weird.
                Constraint::Length(self.header_height),
                Constraint::Length(self.center_height),
                Constraint::Min(self.bottom_height)
                
            ])
            .split(area);

        // Must be done this way. Defining in if will make them look undefined to the compiler. Else is useless for that reason.
        // Must be mut to be changed in if.        
        let mut header = vertical[0];
        let mut main = vertical[1];
        let mut graph_panel = vertical[2];
        
        if self.put_insights_on_top {
            header = vertical[0];
            graph_panel = vertical[1];
            main = vertical[2];
        }

        /** Split main into left and right panels
        You can make the constraints use lenght and min as well and you can do it the same way as the vertical split. The variables are not created though. Here's an example of how you could do it: Constraint::Length(10), Constraint::Min(5),
        This will still apply based on direction, even if panels are inverted! **/
        let horizontal = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_width_percent),
                Constraint::Percentage(100 - self.left_width_percent)
            ])
            .split(main);

        // This is a bit confusing because I don't want to change the variable names, but when it's inverted, the right panel is actually the left panel and the left panel is actually the right panel.
        let mut left_panel = horizontal[0];
        let mut right_panel = horizontal[1];
        
        if self.invert_horizontal_split {
            right_panel = horizontal[0];
            left_panel = horizontal[1];
        }


        /** Split right panel into detail and graph
        Same as for horizontal 
        This will still apply based on direction, even if panels are inverted! **/
        let right_split = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(self.side_vertical_split_percent),
                Constraint::Percentage(100 - self.side_vertical_split_percent),
            ])
            .split(right_panel);
        let mut detail_panel = right_split[0];
        let mut bottom = right_split[1];
        if self.invert_side_vertical_split {
            // This is a bit confusing because I don't want to change the variable names, but when it's inverted, the top panel is actually the bottom panel and the bottom panel is actually the top panel.
            detail_panel = right_split[1];
            bottom = right_split[0]; 
        }

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
