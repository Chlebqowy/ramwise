//! Modern Material Design inspired theme

#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;

enum Ansi {
    Black = 0,
    DarkRed = 1,
    DarkGreen = 2,
    DarkYellow = 3,
    DarkBlue = 4,
    DarkPurple = 5,
    DarkAqua = 6,
    LightGray = 7,
    Gray = 8,
    Red = 9,
    Green = 10,
    Yellow = 11,
    Blue = 12,
    Purple = 13,
    Aqua = 14,
    White = 15,
}

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

    // Memory usage gradient (smooth transitions)
    pub mem_excellent_dim: Color, // < 30%
    pub mem_good_dim: Color,      // 30-50%
    pub mem_moderate_dim: Color,  // 50-70%
    pub mem_high_dim: Color,      // 70-85%
    pub mem_critical_dim: Color,  // > 85%

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
    pub border_subtle: Color,
    pub chosen: String,
    pub use_bar_gradient: bool,
    pub use_mem_color: bool,
    pub process_list_bar_type: self.BarType,

    // Chars
    pub bar_partial_chars: Vec<char>,
    pub bar_full_char: String,
    pub bar_empty_char: String,
    pub bar_all_chars: Vec<char>,
    pub fancy_bar_chars: [char; 4],

    // [Start empty, start full, end empty, end full]
    pub header_dot: String,   // ·
    pub checkmark: String,    //  ✓
    pub triangle: String,     //▲
    pub full_circle: String,  //●
    pub empty_circle: String, //○
    pub semicircle: String,   //◐
    pub diamond: String,      //◈

    // Detail panel chars (state_chip)
    pub running: String,
    pub sleeping: String,
    pub disk_wait: String,
    pub zombie: String,
    pub stopped: String,
    pub tracing: String,
    pub idle: String,
    pub unknown: String,
}

impl Theme {
    enum BarType {
        Standard,
        Gradient,
        Fancy,
        FancyGradient,
    }
    /// Modern dark theme - Material Design 3 inspired
    pub fn dark() -> Self {
        let mem_excellent_vec: Vec<f32> = vec![100.0, 230.0, 140.0];
        let mem_good_vec: Vec<f32> = vec![140.0, 220.0, 100.0];
        let mem_moderate_vec: Vec<f32> = vec![255.0, 210.0, 80.0];
        let mem_high_vec: Vec<f32> = vec![255.0, 150.0, 80.0];
        let mem_critical_vec: Vec<f32> = vec![255.0, 90.0, 90.0];
        let dim_mult: f32 = 0.75;

        let bar_full_char: char = '█';
        let bar_empty_char: char = '░';
        let bar_partial_chars: Vec<char> = vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
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
            mem_excellent: Color::Rgb(
                mem_excellent_vec[0] as u8,
                mem_excellent_vec[1] as u8,
                mem_excellent_vec[2] as u8,
            ), // Fresh green
            mem_good: Color::Rgb(
                mem_good_vec[0] as u8,
                mem_good_vec[1] as u8,
                mem_good_vec[2] as u8,
            ), // Lime
            mem_moderate: Color::Rgb(
                mem_moderate_vec[0] as u8,
                mem_moderate_vec[1] as u8,
                mem_moderate_vec[2] as u8,
            ), // Golden yellow
            mem_high: Color::Rgb(
                mem_high_vec[0] as u8,
                mem_high_vec[1] as u8,
                mem_high_vec[2] as u8,
            ), // Orange
            mem_critical: Color::Rgb(
                mem_critical_vec[0] as u8,
                mem_critical_vec[1] as u8,
                mem_critical_vec[2] as u8,
            ), // Coral red

            // Dim memory gradient - Smooth color progression
            mem_excellent_dim: Color::Rgb(
                (mem_excellent_vec[0] * dim_mult) as u8,
                (mem_excellent_vec[1] * dim_mult) as u8,
                (mem_excellent_vec[2] * dim_mult) as u8,
            ), // Fresh green
            mem_good_dim: Color::Rgb(
                (mem_good_vec[0] * dim_mult) as u8,
                (mem_good_vec[1] * dim_mult) as u8,
                (mem_good_vec[2] * dim_mult) as u8,
            ), // Lime
            mem_moderate_dim: Color::Rgb(
                (mem_moderate_vec[0] * dim_mult) as u8,
                (mem_moderate_vec[1] * dim_mult) as u8,
                (mem_moderate_vec[2] * dim_mult) as u8,
            ), // Golden yellow
            mem_high_dim: Color::Rgb(
                (mem_high_vec[0] * dim_mult) as u8,
                (mem_high_vec[1] * dim_mult) as u8,
                (mem_high_vec[2] * dim_mult) as u8,
            ), // Orange
            mem_critical_dim: Color::Rgb(
                (mem_critical_vec[0] * dim_mult) as u8,
                (mem_critical_vec[1] * dim_mult) as u8,
                (mem_critical_vec[2] * dim_mult) as u8,
            ), // Coral red

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
            chosen: "▶".to_string(),
            use_bar_gradient: true,
            use_mem_color: true,
            process_list_bar_type: self.BarType::FancyBarGradient,
            
            bar_partial_chars,
            bar_full_char: bar_full_char.to_string(),
            bar_empty_char: bar_empty_char.to_string(),
            bar_all_chars: vec![bar_full_char].append(&mut bar_partial_chars.clone()).push(bar_empty_char),
            fancy_bar_chars: ['<', '<', '>', '>'],

            header_dot: "·".to_string(),
            checkmark: "✓".to_string(),
            triangle: "▲".to_string(),
            full_circle: "●".to_string(),
            empty_circle: "○".to_string(),
            semicircle: "◐".to_string(),
            diamond: "◈".to_string(),

            // Detail panel chars
            running: "●".to_string(),
            sleeping: "○".to_string(),
            disk_wait: "◐".to_string(),
            zombie: "✕".to_string(),
            stopped: "◼".to_string(),
            tracing: "◻".to_string(),
            idle: "◌".to_string(),
            unknown: "?".to_string(),

            border_subtle: Color::Rgb(255, 0, 0), // Not subtle, but I prefer it
        }
    }
    pub fn ansi4bit() -> Self {
        let bar_full_char = '█';
        let bar_empty_char = '-';
        let bar_partial_chars = vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
        Self {
            // Base - Basic monochrome l
            bg: Color::Indexed(Ansi::Black as u8),
            fg: Color::Indexed(Ansi::White as u8),
            fg_dim: Color::Indexed(Ansi::LightGray as u8),
            fg_muted: Color::Indexed(Ansi::LightGray as u8),

            // Primary - Aqua
            primary: Color::Indexed(Ansi::Aqua as u8),
            primary_dim: Color::Indexed(Ansi::DarkAqua as u8),

            // Secondary - Blue
            secondary: Color::Indexed(Ansi::Blue as u8),

            // Tertiary - Green
            tertiary: Color::Indexed(Ansi::Green as u8),

            // Semantic
            error: Color::Indexed(Ansi::Red as u8),
            warning: Color::Indexed(Ansi::DarkYellow as u8),
            success: Color::Indexed(Ansi::DarkGreen as u8),
            info: Color::Indexed(Ansi::DarkBlue as u8),

            // Memory gradient - Smooth color progression
            mem_excellent: Color::Indexed(Ansi::DarkGreen as u8), // Fresh green
            mem_good: Color::Indexed(Ansi::Green as u8),          // Lime
            mem_moderate: Color::Indexed(Ansi::DarkYellow as u8), // Golden yellow
            mem_high: Color::Indexed(Ansi::DarkRed as u8),        // Orange
            mem_critical: Color::Indexed(Ansi::Red as u8),        // Coral red

            // Dim memory gradient - Smooth color progression
            mem_excellent_dim: Color::Indexed(Ansi::DarkGreen as u8), // Fresh green
            mem_good_dim: Color::Indexed(Ansi::DarkGreen as u8),      // Lime
            mem_moderate_dim: Color::Indexed(Ansi::DarkYellow as u8), // Golden yellow
            mem_high_dim: Color::Indexed(Ansi::DarkRed as u8),        // Orange
            mem_critical_dim: Color::Indexed(Ansi::DarkRed as u8),    // Coral red

            // UI elements - blue borders and gray selections
            border: Color::Indexed(Ansi::DarkBlue as u8),
            border_focused: Color::Indexed(Ansi::Aqua as u8),
            selection_bg: Color::Indexed(Ansi::Gray as u8),
            selection_fg: Color::Indexed(Ansi::Green as u8),
            header_bg: Color::Indexed(Ansi::Gray as u8),

            // Graph
            graph_line: Color::Indexed(Ansi::Aqua as u8),
            //graph_bg: Color::Rgb(255,0,0),
            graph_border: Color::Indexed(Ansi::DarkAqua as u8), //not actually border
            graph_marker: Marker::Braille, //dot looks better but braille shows data better
            // Process list
            //row_alt_bg: Color::Rgb(22, 22, 30),
            rank_top: Color::Indexed(Ansi::Yellow as u8), // Yellow for #1
            rank_high: Color::Indexed(Ansi::DarkYellow as u8), // dark yellow for top 3
            rank_top_symbol: "* ".to_string(),
            rank_high_symbol: "° ".to_string(),
            chosen: "▶".to_string(),
            use_bar_gradient: true,
            use_mem_color: false,
            process_list_bar_type: self.BarType::Gradient,

            // Use basic chars as a stylistic choice
            bar_partial_chars,
            bar_full_char: bar_full_char.to_string(),
            bar_empty_char: bar_empty_char.to_string(),
            bar_all_chars: vec![bar_full_char].append(&mut bar_partial_chars.clone()).push(bar_empty_char),
            fancy_bar_chars: ['<', '<', '>', '>'],

            header_dot: "·".to_string(),
            checkmark: "✓".to_string(),
            triangle: "▲".to_string(),
            full_circle: "●".to_string(),
            empty_circle: "○".to_string(),
            semicircle: "◐".to_string(),
            diamond: "◈".to_string(),

            // Detail panel chars
            running: "●".to_string(),
            sleeping: "○".to_string(),
            disk_wait: "◐".to_string(),
            zombie: "✕".to_string(),
            stopped: "◼".to_string(),
            tracing: "◻".to_string(),
            idle: "◌".to_string(),
            unknown: "?".to_string(),

            border_subtle: Color::Indexed(Ansi::Red as u8), // Not subtle, but I prefer it
        }
    }
    // Paper-like light theme "inspired" (mostly taken 1:1) by GruvBox Light
    pub fn light() -> Self {
        let mem_excellent_vec: Vec<f32> = vec![100.0, 230.0, 140.0];
        let mem_good_vec: Vec<f32> = vec![0x98 as f32, 0x97 as f32, 0x1a as f32];
        let mem_moderate_vec: Vec<f32> = vec![0xf7 as f32, 0x99 as f32, 0x21 as f32];
        let mem_high_vec: Vec<f32> = vec![0xfe as f32, 0x80 as f32, 0x19 as f32];
        let mem_critical_vec: Vec<f32> = vec![0xaf as f32, 0x3a as f32, 0x03 as f32];
        let dim_mult: f32 = 0.9;

        let bar_full_char = '█';
        let bar_empty_char = '░';
        let bar_partial_chars = vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
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
            mem_excellent: Color::Rgb(
                mem_excellent_vec[0] as u8,
                mem_excellent_vec[1] as u8,
                mem_excellent_vec[2] as u8,
            ), // Fresh green - not gruvbox, their green is too lime
            mem_good: Color::Rgb(
                mem_good_vec[0] as u8,
                mem_good_vec[1] as u8,
                mem_good_vec[2] as u8,
            ), // Lime
            mem_moderate: Color::Rgb(
                mem_moderate_vec[0] as u8,
                mem_moderate_vec[1] as u8,
                mem_moderate_vec[2] as u8,
            ), // Middle yellow
            mem_high: Color::Rgb(
                mem_high_vec[0] as u8,
                mem_high_vec[1] as u8,
                mem_high_vec[2] as u8,
            ), // Light orange
            mem_critical: Color::Rgb(
                mem_critical_vec[0] as u8,
                mem_critical_vec[1] as u8,
                mem_critical_vec[2] as u8,
            ), // Dark orange

            // Memory gradient - Smooth color progression - modifiedbased on vibes, not directly from gruvbox
            mem_excellent_dim: Color::Rgb(
                (mem_excellent_vec[0] * dim_mult) as u8,
                (mem_excellent_vec[1] * dim_mult) as u8,
                (mem_excellent_vec[2] * dim_mult) as u8,
            ),
            mem_good_dim: Color::Rgb(
                (mem_good_vec[0] * dim_mult) as u8,
                (mem_good_vec[1] * dim_mult) as u8,
                (mem_good_vec[2] * dim_mult) as u8,
            ),
            mem_moderate_dim: Color::Rgb(
                (mem_moderate_vec[0] * dim_mult) as u8,
                (mem_moderate_vec[1] * dim_mult) as u8,
                (mem_moderate_vec[2] * dim_mult) as u8,
            ),
            mem_high_dim: Color::Rgb(
                (mem_high_vec[0] * dim_mult) as u8,
                (mem_high_vec[1] * dim_mult) as u8,
                (mem_high_vec[2] * dim_mult) as u8,
            ),
            mem_critical_dim: Color::Rgb(
                (mem_critical_vec[0] * dim_mult) as u8,
                (mem_critical_vec[1] * dim_mult) as u8,
                (mem_critical_vec[2] * dim_mult) as u8,
            ),

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
            chosen: "▶".to_string(),
            use_bar_gradient: false,
            use_mem_color: true,
            process_list_bar_type: self.BarType::Standard,

            bar_partial_chars,
            bar_full_char: bar_full_char.to_string(),
            bar_empty_char: bar_empty_char.to_string(),
            bar_all_chars: vec![bar_full_char].append(&mut bar_partial_chars.clone()).push(bar_empty_char),
            fancy_bar_chars: ['<', '<', '>', '>'],
            header_dot: "·".to_string(),
            checkmark: "✓".to_string(),
            triangle: "▲".to_string(),
            full_circle: "●".to_string(),
            empty_circle: "○".to_string(),
            semicircle: "◐".to_string(),
            diamond: "◈".to_string(),

            // Detail panel chars
            running: "●".to_string(),
            sleeping: "○".to_string(),
            disk_wait: "◐".to_string(),
            zombie: "✕".to_string(),
            stopped: "◼".to_string(),
            tracing: "◻".to_string(),
            idle: "◌".to_string(),
            unknown: "?".to_string(),

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

    pub fn mem_color_dim(&self, percent: f64) -> Color {
        if percent >= 85.0 {
            self.mem_critical_dim
        } else if percent >= 70.0 {
            self.mem_high_dim
        } else if percent >= 50.0 {
            self.mem_moderate_dim
        } else if percent >= 30.0 {
            self.mem_good_dim
        } else {
            self.mem_excellent_dim
        }
    }

    /// Get memory color with interpolation for smooth gradients
    pub fn mem_color_interpolated(&self, percent: f64) -> Color {
        self.mem_color(percent)
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

    pub fn generate_gradient(segments: u8, color1: Color, color2: Color) -> Vec<Color> {
        if let Color::Rgb(r1, g1, b1) = color1
            && let Color::Rgb(r2, g2, b2) = color2
        {
            let mut result = Vec::with_capacity(segments as usize);
            for segment in 0..segments {
                let progress = if segments <= 1 {
                    0.0
                } else {
                    segment as f64 / (segments - 1) as f64
                };
                let (r, g, b) = lerp_rgb((r1, g1, b1), (r2, g2, b2), progress);
                result.push(Color::Rgb(r, g, b));
            }
            result
        } else {
            let mut result = vec![];
            let color1_segments = if segments % 2 == 1 {
                segments / 2 + 1
            } else {
                segments / 2
            };
            let color2_segments = if segments % 2 == 1 {
                color1_segments - 1
            } else {
                segments / 2
            }; // did the if just to be safe if it rounds up .5
            for _segment in 0..color1_segments {
                result.push(color1);
            }
            for _segment in 0..color2_segments {
                result.push(color2);
            }
            result
        }
    }

    pub fn create_gradient_bar(
        bar: String,
        bar_width: usize,
        color1: Color,
        color2: Color,
        spans: &mut Vec<Span>,
    ) {
        let bar_gradient = Theme::generate_gradient(bar_width as u8, color1, color2);
        let bar_char_array: Vec<char> = bar.chars().collect();
        for index in 0..bar_width {
            let bar_char = bar_char_array[index];
            let bar_char_style = Style::default().fg(bar_gradient[index]);
            let span = Span::styled(bar_char.to_string(), bar_char_style);
            spans.push(span); // make this a toggle 
        }
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
    pub fn create_fancy_bar(&self, percent: f64, width: usize)
    {
        let mut bar: Vec<char> = self.create_sleek_bar(percent, width).chars().collect();
        bar[0] = {
            let pos = self.bar_all_chars.position(bar[0]) as f32;
            let len = self.bar_all_chars.len() as f32;
            let is_full = pos/len;
            if is_full >= 0.5 { self.fancy_bar_chars[0] } else { self.fancy_bar_chars[1]} 
        }
        let last: usize = bar.len() - 1 as usize;
        bar[last] = {
            let pos = self.bar_all_chars.position(bar[last]) as f32;
            let len = self.bar_all_chars.len() as f32;
            let is_full = pos/len;
            if is_full >= 0.5 { self.fancy_bar_chars[2] } else { self.fancy_bar_chars[3]} 
        }

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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_gradient() {
        let color1 = Color::Rgb(0, 0, 255);
        let color2 = Color::Rgb(255, 0, 0);
        let gradient = Theme::generate_gradient(8, color1, color2);
        assert_eq!(gradient.len(), 8);
        assert_eq!(gradient[0], color1);
        assert_eq!(gradient[7], color2);
        assert_ne!(gradient[3], gradient[4]);
    }

    #[test]
    fn check_gradient_long() {
        let color1 = Color::Rgb(0, 0, 255);
        let color2 = Color::Rgb(255, 0, 0);
        let gradient = Theme::generate_gradient(80, color1, color2);
        assert_eq!(gradient.len(), 80);
        assert_eq!(gradient[0], color1);
        assert_eq!(gradient[79], color2);
        assert_ne!(gradient[30], gradient[40]);
    }

    #[test]
    fn check_0_1_gradient() {
        let color1 = Color::Rgb(0, 0, 255);
        let color2 = Color::Rgb(255, 0, 0);
        assert_eq!(Theme::generate_gradient(0, color1, color2), vec![]);
        assert_eq!(Theme::generate_gradient(1, color1, color2), vec![color1]);
    }

    #[test]
    fn check_non_rgb_gradient() {
        let gradient = Theme::generate_gradient(7, Color::Blue, Color::Red);
        assert_eq!(
            gradient,
            vec![
                Color::Blue,
                Color::Blue,
                Color::Blue,
                Color::Blue,
                Color::Red,
                Color::Red,
                Color::Red,
            ]
        );
    }

    #[test]
    fn check_create_gradient_bar_color() {
        let mut spans = Vec::new();
        Theme::create_gradient_bar(
            "abcdefg".to_string(),
            7,
            Color::Blue,
            Color::Red,
            &mut spans,
        );
        assert_eq!(spans[0].style.fg, Some(Color::Blue));
        assert_eq!(spans[3].style.fg, Some(Color::Blue));
        assert_eq!(spans[4].style.fg, Some(Color::Red));
        assert_eq!(spans[6].style.fg, Some(Color::Red));
    }
    #[test]
    fn check_create_gradient_bar_color_long() {
        let mut spans = Vec::new();
        Theme::create_gradient_bar(
            "1234567890abcdefghijklmnopqrstuwvxyz".to_string(),
            36,
            Color::Blue,
            Color::Red,
            &mut spans,
        );
        assert_eq!(spans[0].style.fg, Some(Color::Blue));
        assert_eq!(spans[10].style.fg, Some(Color::Blue));
        assert_eq!(spans[17].style.fg, Some(Color::Blue));
        assert_eq!(spans[18].style.fg, Some(Color::Red));
        assert_eq!(spans[30].style.fg, Some(Color::Red));
        assert_eq!(spans[35].style.fg, Some(Color::Red));
    }
    #[test]
    fn check_create_gradient_bar_char() {
        let mut spans = Vec::new();
        Theme::create_gradient_bar(
            "abcdefg".to_string(),
            7,
            Color::Blue,
            Color::Red,
            &mut spans,
        );

        assert_eq!(spans[0].content, "a");
        assert_eq!(spans[3].content, "d");
        assert_eq!(spans[4].content, "e");
        assert_eq!(spans[6].content, "g");
    }
    #[test]
    fn check_create_gradient_bar_char_long() {
        let mut spans = Vec::new();
        Theme::create_gradient_bar(
            "1234567890abcdefghijklmnopqrstuwvxyz".to_string(),
            36,
            Color::Blue,
            Color::Red,
            &mut spans,
        ); // 36 chars

        assert_eq!(spans[0].content, "1");
        assert_eq!(spans[9].content, "0");
        assert_eq!(spans[15].content, "f");
        assert_eq!(spans[35].content, "z");
    }
    #[test]
    fn check_create_sleek_bar_whole() {
        let theme = Theme::dark();
        let part: f64 = 100.0;
        let chars: usize = 8;
        let full_chars: f64 = chars as f64 * part / 100.0;
        let empty_chars: f64 = chars as f64 * (1 as f64 - part / 100.0);
        let correct_result = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", correct_result);
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_eq!(result, correct_result);
    }
    #[test]
    fn check_create_sleek_bar_half() {
        let theme = Theme::dark();
        let part: f64 = 50.0;
        let chars: usize = 8;
        let full_chars: f64 = chars as f64 * part / 100.0;
        let empty_chars: f64 = chars as f64 * (1 as f64 - part / 100.0);
        let correct_result = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", correct_result);
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_eq!(result, correct_result);
    }
    #[test]
    fn check_create_sleek_bar_quarter() {
        let theme = Theme::dark();
        let part: f64 = 25.0;
        let chars: usize = 8;
        let full_chars: f64 = chars as f64 * part / 100.0;
        let empty_chars: f64 = chars as f64 * (1 as f64 - part / 100.0);
        let correct_result = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", correct_result);
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_eq!(result, correct_result);
    }
    #[test]
    fn check_create_sleek_bar_eighth() {
        let theme = Theme::dark();
        let part: f64 = 12.5;
        let chars: usize = 8;
        let full_chars: f64 = chars as f64 * part / 100.0;
        let empty_chars: f64 = chars as f64 * (1 as f64 - part / 100.0);
        let correct_result = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", correct_result);
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_eq!(result, correct_result);
    }
    #[test]
    fn check_create_sleek_bar_nonwhole() {
        let theme = Theme::dark();
        let part: f64 = 20.0;
        let chars: usize = 8;
        let full_chars: f64 = chars as f64 * part / 100.0;
        let empty_chars: f64 = chars as f64 * (1 as f64 - part / 100.0);
        let incorrect_result_a = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", incorrect_result_a);
        let incorrect_result_b = theme
            .bar_full_char
            .to_string()
            .repeat(full_chars as usize + 1 as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize - 1 as usize);
        print!("{}", incorrect_result_b);
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_ne!(result, incorrect_result_a);
        assert_ne!(result, incorrect_result_b);
    }
    // maybe add a test for that without middle characters
    #[test]
    fn check_create_sleek_bar_nonwhole_len() {
        let theme = Theme::dark();
        let part: f64 = 20.0;
        let chars: usize = 8;
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_eq!(result.chars().count(), chars);
    }
    #[test]
    fn check_create_sleek_bar_quarter_ansi() {
        let theme = Theme::ansi4bit();
        let part: f64 = 25.0;
        let chars: usize = 8;
        let full_chars: f64 = chars as f64 * part / 100.0;
        let empty_chars: f64 = chars as f64 * (1 as f64 - part / 100.0);
        let correct_result = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", correct_result);
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_eq!(result, correct_result);
    }
    #[test]
    fn check_create_sleek_bar_quarter_big() {
        let theme = Theme::dark();
        let part: f64 = 25.0;
        let chars: usize = 800;
        let full_chars: f64 = chars as f64 * part / 100.0;
        let empty_chars: f64 = chars as f64 * (1 as f64 - part / 100.0);
        let correct_result = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", correct_result);
        let result = Theme::create_sleek_bar(&theme, part, chars);
        assert_eq!(result, correct_result);
    }
}
