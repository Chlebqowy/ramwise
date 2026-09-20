//! Modern Material Design inspired theme

#![allow(dead_code)]

use clap::builder::ValueRange;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;

/// Modern color theme - Material Design 3 inspired
#[derive(Debug, Clone)]
pub struct Theme {
    // Base colors - Deep dark with slight blue undertone
    pub bg: Color,
    pub fg: Color,
    pub fg_dim: Color,
    pub fg_muted: Color,

    // Primary accent - Vibrant purple/violet
    pub primary: Color,
    pub primary_dim: Color,

    // Secondary accent - Teal/Cyan
    pub secondary: Color,

    // Tertiary - Coral/Pink for special highlights (only quit keybind for now)
    pub tertiary: Color,

    // Semantic colors
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,

    // Memory usage gradient (smooth transitions)
    pub mem_excellent: Color, // < 30%
    pub mem_good: Color,      // 30-50%
    pub mem_moderate: Color,  // 50-70%
    pub mem_high: Color,      // 70-85%
    pub mem_critical: Color,  // > 85%

    // UI elements
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub header_bg: Color,

    // Graph colors
    pub graph_line: Color,
    //pub graph_bg: Color, // not implemented in the library, thus unused https://docs.rs/ratatui/latest/ratatui/widgets/struct.Dataset.html#method.style
    pub graph_border: Color,
    pub graph_marker: Marker,

    // Process list specific
    pub rank_top: Color,
    pub rank_high: Color,
    pub rank_top_symbol: String,
    pub rank_high_symbol: String,

    pub bar_partial_chars: Vec<char>,
    pub bar_full_char: String,
    pub bar_empty_char: String,
    pub border_subtle: Color,
}

impl Theme {
    /// Modern dark theme - Material Design 3 inspired
    pub fn dark() -> Self {
        Self {
            // Base - Rich dark with subtle warmth
            bg: Color::Rgb(18, 18, 24),
            fg: Color::Rgb(230, 230, 240),
            fg_dim: Color::Rgb(160, 160, 175),
            fg_muted: Color::Rgb(100, 100, 115),

            // Primary - Electric violet/purple
            primary: Color::Rgb(180, 130, 255),
            primary_dim: Color::Rgb(140, 90, 200),

            // Secondary - Vibrant teal
            secondary: Color::Rgb(80, 220, 200),

            // Tertiary - Warm coral
            tertiary: Color::Rgb(255, 140, 120),

            // Semantic
            error: Color::Rgb(255, 100, 100),
            warning: Color::Rgb(255, 190, 70),
            success: Color::Rgb(100, 230, 140),
            info: Color::Rgb(100, 180, 255),

            // Memory gradient - Smooth color progression
            mem_excellent: Color::Rgb(100, 230, 140), // Fresh green
            mem_good: Color::Rgb(140, 220, 100),      // Lime
            mem_moderate: Color::Rgb(255, 210, 80),   // Golden yellow
            mem_high: Color::Rgb(255, 150, 80),       // Orange
            mem_critical: Color::Rgb(255, 90, 90),    // Coral red

            // UI elements
            border: Color::Rgb(55, 55, 70),
            border_focused: Color::Rgb(180, 130, 255),
            selection_bg: Color::Rgb(60, 40, 90),
            selection_fg: Color::Rgb(255, 255, 255),
            header_bg: Color::Rgb(28, 28, 36),

            // Graph
            graph_line: Color::Rgb(230, 230, 240),
            //graph_bg: Color::Rgb(255,0,0),
            graph_border: Color::Rgb(55, 55, 70), //not actually border
            graph_marker: Marker::Braille,
            // Process list
            //row_alt_bg: Color::Rgb(22, 22, 30),
            rank_top: Color::Rgb(255, 190, 70),   // Gold for #1
            rank_high: Color::Rgb(180, 130, 255), // Purple for top 3
            rank_top_symbol: "● ".to_string(),
            rank_high_symbol: "⊗ ".to_string(),

            bar_partial_chars: vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'],
            bar_full_char: "█".to_string(),
            bar_empty_char: "░".to_string(),

            border_subtle: Color::Rgb(255, 0, 0), // Not subtle, but I prefer it
        }
    }
    // Paper-like light theme "inspired" (mostly taken 1:1) by GruvBox Light
    pub fn light() -> Self {
        Self {
            // Base - Very light gray with a yellow tint
            bg: Color::Rgb(0xfb, 0xf1, 0xc7),
            fg: Color::Rgb(0x28, 0x28, 0x28),
            fg_dim: Color::Rgb(0x3c, 0x38, 0x36),
            fg_muted: Color::Rgb(0x3c, 0x38, 0x36),

            // Primary - GruvBox's aqua (Mix between both color schemes. Actually a slightly blue-ish green)
            primary: Color::Rgb(0x8e, 0xc0, 0x7c),
            primary_dim: Color::Rgb(0x68, 0x9d, 0x6a),

            // Secondary - GruvBox Dark's blue
            secondary: Color::Rgb(0x42, 0x7b, 0x58),

            // Tertiary - Red
            tertiary: Color::Rgb(0xcc, 0x24, 0x1d),

            // Semantic - All gruvbox light colors.
            error: Color::Rgb(0xcc, 0x24, 0x1d),
            warning: Color::Rgb(0xd7, 0x99, 0x21),
            success: Color::Rgb(0x98, 0x97, 0x1a),
            info: Color::Rgb(0x83, 0xa5, 0x98),

            // Memory gradient - Smooth color progression
            mem_excellent: Color::Rgb(100, 230, 140), // Fresh green - not gruvbox, their green is too lime
            mem_good: Color::Rgb(0x98, 0x97, 0x1a),   // Lime
            mem_moderate: Color::Rgb(0xf7, 0x99, 0x21), // Middle yellow
            mem_high: Color::Rgb(0xfe, 0x80, 0x19),   // Light orange
            mem_critical: Color::Rgb(0xaf, 0x3a, 0x03), // Dark orange

            // UI elements - GruvBox aqua and fg/bg, taken from light
            border: Color::Rgb(0x7c, 0x6f, 0x64),
            border_focused: Color::Rgb(0x68, 0x9d, 0x6a),
            selection_bg: Color::Rgb(0x42, 0x7b, 0x58),
            selection_fg: Color::Rgb(0xfb, 0xf1, 0xc7),
            header_bg: Color::Rgb(0xeb, 0xdb, 0xb2),

            // Graph - Grubvox dark aqua
            graph_line: Color::Rgb(0x8e, 0xc0, 0x7c),
            //graph_bg: Color::Rgb(0x3c, 0x38, 0x36),
            graph_border: Color::Rgb(0x3c, 0x38, 0x36), //not actually border
            graph_marker: Marker::Braille,

            // Process list
            rank_top: Color::Rgb(0xfa, 0xbd, 0x2f), // Light yellow for top 1
            rank_high: Color::Rgb(0xd6, 0x5d, 0x0e), // Middle orange for top 3
            rank_top_symbol: "● ".to_string(),
            rank_high_symbol: "⊗ ".to_string(),

            bar_partial_chars: vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'],
            bar_full_char: "█".to_string(),
            bar_empty_char: "░".to_string(),

            border_subtle: Color::Rgb(0xd3, 0x96, 0x9b), // Not subtle, but I prefer it
        }
    }

    /// Get memory color based on percentage (0-100) with smooth gradient
    pub fn mem_color(&self, percent: f64) -> Color {
        if percent >= 85.0 {
            self.mem_critical
        } else if percent >= 70.0 {
            self.mem_high
        } else if percent >= 50.0 {
            self.mem_moderate
        } else if percent >= 30.0 {
            self.mem_good
        } else {
            self.mem_excellent
        }
    }

    /// Get memory color with interpolation for smooth gradients
    pub fn mem_color_interpolated(&self, percent: f64) -> Color {
        // Clamp to 0-100
        let p = percent.clamp(0.0, 100.0);

        // Define color stops
        let (r, g, b) = if p < 30.0 {
            // Excellent zone: fresh green
            (100, 230, 140)
        } else if p < 50.0 {
            // Good zone: transition green -> yellow
            let t = (p - 30.0) / 20.0;
            lerp_rgb((100, 230, 140), (255, 210, 80), t)
        } else if p < 70.0 {
            // Moderate zone: transition yellow -> orange
            let t = (p - 50.0) / 20.0;
            lerp_rgb((255, 210, 80), (255, 150, 80), t)
        } else if p < 85.0 {
            // High zone: transition orange -> red
            let t = (p - 70.0) / 15.0;
            lerp_rgb((255, 150, 80), (255, 90, 90), t)
        } else {
            // Critical: red
            (255, 90, 90)
        };

        Color::Rgb(r, g, b)
    }

    /// Get severity color
    pub fn severity_color(&self, severity: crate::analyzer::Severity) -> Color {
        match severity {
            crate::analyzer::Severity::Critical => self.error,
            crate::analyzer::Severity::Warning => self.warning,
            crate::analyzer::Severity::Info => self.info,
        }
    }

    // === Style builders ===

    pub fn base_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.fg_dim)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.fg_muted)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .bg(self.header_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selection_fg)
            .bg(self.selection_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused)
        } else {
            Style::default().fg(self.border)
        }
    }

    pub fn critical_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Style for ranking badges
    pub fn rank_style(&self, rank: usize) -> Style {
        let color = match rank {
            0 => self.rank_top,
            1..=2 => self.rank_high,
            _ => Color::Rgb(255, 0, 0), //should be empty under normal conditions, set to red
        };
        Style::default().fg(color)
    }
    /// Create a sleek progress bar with partial block characters
    pub fn create_sleek_bar(&self, percent: f64, width: usize) -> String {
        let total_eighths =
            ((percent / 100.0) * (width * self.bar_partial_chars.len()) as f64).round() as usize;
        let full_blocks = total_eighths / self.bar_partial_chars.len();
        let partial = total_eighths % self.bar_partial_chars.len();

        let mut bar = self.bar_full_char.repeat(full_blocks);

        if partial > 0 && full_blocks < width {
            bar.push(self.bar_partial_chars[partial]);
        }

        let remaining = width.saturating_sub(bar.chars().count());
        bar.push_str(&self.bar_empty_char.repeat(remaining));

        bar
    }
}
/// Linear interpolation between two RGB colors
fn lerp_rgb(from: (u8, u8, u8), to: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        (from.0 as f64 + (to.0 as f64 - from.0 as f64) * t) as u8,
        (from.1 as f64 + (to.1 as f64 - from.1 as f64) * t) as u8,
        (from.2 as f64 + (to.2 as f64 - from.2 as f64) * t) as u8,
    )
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
