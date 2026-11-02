//! Modern Material Design inspired theme

#![allow(dead_code)]

use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

use super::theme_source::ThemeSource;

/// Files the Hyprland theme is read from, in lookup order
pub(crate) fn hyprland_theme_paths() -> Vec<std::path::PathBuf> {
    hyprland::candidate_paths()
}

/// Accepts `[r, g, b]`, an ANSI index (`0`-`255`) or a string such as
/// `"#rrggbb"`, `"red"`, `"lightblue"` or `"12"`.
fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawColor {
        Rgb([u8; 3]),
        Indexed(u8),
        Named(String),
    }
    match RawColor::deserialize(deserializer)? {
        RawColor::Rgb([r, g, b]) => Ok(Color::Rgb(r, g, b)),
        RawColor::Indexed(index) => Ok(Color::Indexed(index)),
        RawColor::Named(name) => Color::from_str(&name)
            .map_err(|_| serde::de::Error::custom(format!("invalid color: {name}"))),
    }
}

/// Writes colors in a form `deserialize_color` reads back
fn serialize_color<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match color {
        Color::Rgb(r, g, b) => serializer.serialize_str(&format!("#{r:02x}{g:02x}{b:02x}")),
        Color::Indexed(index) => serializer.serialize_u8(*index),
        other => serializer.serialize_str(&other.to_string().to_lowercase()),
    }
}

/// Writes markers in a form `deserialize_marker` reads back
fn serialize_marker<S>(marker: &Marker, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let name = match marker {
        Marker::Dot => "dot".to_string(),
        Marker::Block => "block".to_string(),
        Marker::Bar => "bar".to_string(),
        Marker::Braille => "braille".to_string(),
        Marker::HalfBlock => "half_block".to_string(),
        Marker::Quadrant => "quadrant".to_string(),
        Marker::Sextant => "sextant".to_string(),
        Marker::Octant => "octant".to_string(),
        Marker::Custom(c) => c.to_string(),
        _ => "braille".to_string(),
    };
    serializer.serialize_str(&name)
}

/// Accepts a marker name (`dot`, `block`, `bar`, `braille`, `half_block`,
/// `quadrant`, `sextant`, `octant`) or any single character for a custom marker.
fn deserialize_marker<'de, D>(deserializer: D) -> Result<Marker, D::Error>
where
    D: Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    match name.to_lowercase().replace(['-', '_'], "").as_str() {
        "dot" => Ok(Marker::Dot),
        "block" => Ok(Marker::Block),
        "bar" => Ok(Marker::Bar),
        "braille" => Ok(Marker::Braille),
        "halfblock" => Ok(Marker::HalfBlock),
        "quadrant" => Ok(Marker::Quadrant),
        "sextant" => Ok(Marker::Sextant),
        "octant" => Ok(Marker::Octant),
        _ => {
            let mut chars = name.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(Marker::Custom(c)),
                _ => Err(serde::de::Error::custom(format!(
                    "invalid graph marker: {name}"
                ))),
            }
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)] // not sure what this does but the error told me
#[serde(rename_all = "snake_case")]
pub enum BarType {
    Standard,
    Gradient,
    Fancy,
    FancyGradient,
}

/// How swap usage is shown in the header
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SwapDisplay {
    /// A usage bar, drawn like the RAM bar (uses `header_bar_type`)
    Bar,
    /// A single status icon (`empty_circle` / `full_circle` / `triangle`)
    Icon,
}

/// Modern color theme - Material Design 3 inspired
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    // Base colors - Deep dark with slight blue undertone
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub bg: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub fg: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub fg_dim: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub fg_muted: Color,

    // Primary accent - Vibrant purple/violet
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub primary: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub primary_dim: Color,

    // Secondary accent - Teal/Cyan
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub secondary: Color,

    // Tertiary - Coral/Pink for special highlights (only quit keybind for now)
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub tertiary: Color,

    // Semantic colors
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub error: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub warning: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub success: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub info: Color,

    // Memory usage gradient (smooth transitions)
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_excellent: Color, // < 30%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_good: Color, // 30-50%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_moderate: Color, // 50-70%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_high: Color, // 70-85%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_critical: Color, // > 85%

    // Memory usage gradient (smooth transitions)
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_excellent_dim: Color, // < 30%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_good_dim: Color, // 30-50%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_moderate_dim: Color, // 50-70%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_high_dim: Color, // 70-85%
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub mem_critical_dim: Color, // > 85%

    // UI elements
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub border: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub border_focused: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub selection_bg: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub selection_fg: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub header_bg: Color,

    // Graph colors
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub graph_line: Color,
    //pub graph_bg: Color, // not implemented in the library, thus unused https://docs.rs/ratatui/latest/ratatui/widgets/struct.Dataset.html#method.style
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub graph_border: Color,
    #[serde(
        deserialize_with = "deserialize_marker",
        serialize_with = "serialize_marker"
    )]
    pub graph_marker: Marker,

    // Process list specific
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub rank_top: Color,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub rank_high: Color,
    pub rank_top_symbol: String,
    pub rank_high_symbol: String,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub border_subtle: Color,
    pub chosen: String,
    pub use_mem_color: bool,
    /// Process list bars fill up at this share of total RAM (in %), or at the
    /// biggest process if that uses more
    pub process_list_scale_percent: f64,
    pub ignored_process_indicator: char,

    pub process_list_bar_type: BarType,
    pub detail_panel_bar_type: BarType,
    pub header_bar_type: BarType,
    pub swap_display: SwapDisplay,

    // Chars
    pub bar_partial_chars: Vec<char>,
    pub bar_full_char: String,
    pub bar_empty_char: String,
    /// Derived from the empty, partial and full bar chars; not read from config
    #[serde(skip)]
    pub bar_all_chars: Vec<char>,
    pub fancy_bar_chars: [char; 4],

    // [Start empty, start full, end empty, end full]
    pub header_dot: String,    // ·
    pub checkmark: String,     //  ✓
    pub triangle: String,      //▲
    pub full_circle: String,   //●
    pub empty_circle: String,  //○
    pub semicircle: String,    //◐
    pub diamond: String,       //◈
    pub empty_diamond: String, //◇
    pub arrow_up: String,      //↑
    pub arrow_right: String,   //→
    pub ellipsis: String,      //…
    pub loading: String,       //◌

    // Detail panel chars (state_chip)
    pub running: String,
    pub sleeping: String,
    pub disk_wait: String,
    pub zombie: String,
    pub stopped: String,
    pub tracing: String,
    pub idle: String,
    pub unknown: String,

    // Title and label icons; an empty string shows no icon
    pub app_icon: String,
    pub ram_icon: String,
    pub swap_icon: String,
    pub process_list_icon: String,
    pub insights_icon: String,
    pub graph_icon: String,
    pub memory_icon: String,
    pub breakdown_icon: String,

    // Layout
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

impl Theme {
    /// Modern dark theme - Material Design 3 inspired
    pub fn dark() -> Self {
        let mem_excellent_vec: Vec<f32> = vec![100.0, 230.0, 140.0];
        let mem_good_vec: Vec<f32> = vec![140.0, 220.0, 100.0];
        let mem_moderate_vec: Vec<f32> = vec![255.0, 210.0, 80.0];
        let mem_high_vec: Vec<f32> = vec![255.0, 150.0, 80.0];
        let mem_critical_vec: Vec<f32> = vec![255.0, 90.0, 90.0];
        let dim_mult: f32 = 0.75;

        let bar_full_char_pre: char = '█';
        let bar_empty_char_pre: char = '░';
        //let bar_empty_char_pre: char = '·'; // recommend changing for this if you want fancy bar
        let bar_partial_chars: Vec<char> = vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

        let bar_full_char: String = bar_full_char_pre.to_string();
        let bar_empty_char: String = bar_empty_char_pre.to_string();
        let mut bar_all_chars: Vec<char> = vec![bar_empty_char_pre];
        bar_all_chars.extend(bar_partial_chars.iter().cloned());
        bar_all_chars.push(bar_full_char_pre);
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
            use_mem_color: true,
            ignored_process_indicator: '-',
            process_list_scale_percent: 50.0,

            //process_list_bar_type: BarType::FancyGradient,
            process_list_bar_type: BarType::Gradient,
            detail_panel_bar_type: BarType::Gradient,
            //header_bar_type: BarType::FancyGradient,
            header_bar_type: BarType::Gradient,
            swap_display: SwapDisplay::Bar,
            // only change to fancy if you have a nerd font or changed fancy_bar_chars
            bar_partial_chars,
            bar_full_char,
            bar_empty_char,
            bar_all_chars,
            //fancy_bar_chars: ['<', '<', '>', '>'],
            fancy_bar_chars: ['', '', '', ''],

            header_dot: "·".to_string(),
            checkmark: "✓".to_string(),
            triangle: "▲".to_string(),
            full_circle: "●".to_string(),
            empty_circle: "○".to_string(),
            semicircle: "◐".to_string(),
            diamond: "◈".to_string(),
            empty_diamond: "◇".to_string(),
            arrow_up: "↑".to_string(),
            arrow_right: "→".to_string(),
            ellipsis: "…".to_string(),
            loading: "◌".to_string(),

            // Detail panel chars
            running: "●".to_string(),
            sleeping: "○".to_string(),
            disk_wait: "◐".to_string(),
            zombie: "✕".to_string(),
            stopped: "◼".to_string(),
            tracing: "◻".to_string(),
            idle: "◌".to_string(),
            unknown: "?".to_string(),

            app_icon: String::new(),
            ram_icon: String::new(),
            swap_icon: String::new(),
            process_list_icon: String::new(),
            insights_icon: String::new(),
            graph_icon: "◈".to_string(),
            memory_icon: String::new(),
            breakdown_icon: String::new(),

            border_subtle: Color::Rgb(255, 0, 0), // Not subtle, but I prefer it

            // Layout
            header_height: 1,
            center_height: 10,
            bottom_height: 4,
            left_width_percent: 40,
            side_vertical_split_percent: 60,

            invert_horizontal_split: false,
            invert_side_vertical_split: false,
            put_insights_on_top: false,
        }
    }
    pub fn ansi4bit() -> Self {
        let bar_full_char_pre = '█';
        let bar_empty_char_pre = '-';
        let bar_partial_chars = vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

        let bar_full_char: String = bar_full_char_pre.to_string();
        let bar_empty_char: String = bar_empty_char_pre.to_string();
        let mut bar_all_chars: Vec<char> = vec![bar_empty_char_pre];
        bar_all_chars.extend(bar_partial_chars.iter().cloned());
        bar_all_chars.push(bar_full_char_pre);
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
            use_mem_color: false,
            ignored_process_indicator: '-',
            process_list_scale_percent: 50.0,
            process_list_bar_type: BarType::Gradient,
            detail_panel_bar_type: BarType::Gradient,
            header_bar_type: BarType::Gradient,
            swap_display: SwapDisplay::Bar,

            // Use basic chars as a stylistic choice
            bar_partial_chars,
            bar_full_char,
            bar_empty_char,
            bar_all_chars,
            fancy_bar_chars: [bar_empty_char_pre, bar_full_char_pre, '>', '>'],

            header_dot: "·".to_string(),
            checkmark: "✓".to_string(),
            triangle: "▲".to_string(),
            full_circle: "●".to_string(),
            empty_circle: "○".to_string(),
            semicircle: "◐".to_string(),
            diamond: "◈".to_string(),
            empty_diamond: "◇".to_string(),
            arrow_up: "↑".to_string(),
            arrow_right: "→".to_string(),
            ellipsis: "…".to_string(),
            loading: "◌".to_string(),

            // Detail panel chars
            running: "●".to_string(),
            sleeping: "○".to_string(),
            disk_wait: "◐".to_string(),
            zombie: "✕".to_string(),
            stopped: "◼".to_string(),
            tracing: "◻".to_string(),
            idle: "◌".to_string(),
            unknown: "?".to_string(),

            app_icon: String::new(),
            ram_icon: String::new(),
            swap_icon: String::new(),
            process_list_icon: String::new(),
            insights_icon: String::new(),
            graph_icon: "◈".to_string(),
            memory_icon: String::new(),
            breakdown_icon: String::new(),

            border_subtle: Color::Indexed(Ansi::Red as u8), // Not subtle, but I prefer it

            // Layout
            header_height: 1,
            center_height: 10,
            bottom_height: 4,
            left_width_percent: 40,
            side_vertical_split_percent: 60,

            invert_horizontal_split: false,
            invert_side_vertical_split: false,
            put_insights_on_top: false,
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

        let bar_full_char_pre = '█';
        let bar_empty_char_pre = '░';
        let bar_partial_chars = vec!['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

        let bar_full_char: String = bar_full_char_pre.to_string();
        let bar_empty_char: String = bar_empty_char_pre.to_string();
        let mut bar_all_chars: Vec<char> = vec![bar_empty_char_pre];
        bar_all_chars.extend(bar_partial_chars.iter().cloned());
        bar_all_chars.push(bar_full_char_pre);
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
            use_mem_color: true,
            ignored_process_indicator: '-',
            process_list_scale_percent: 50.0,
            process_list_bar_type: BarType::Standard,
            detail_panel_bar_type: BarType::Standard,
            header_bar_type: BarType::Standard,
            swap_display: SwapDisplay::Bar,

            bar_partial_chars,
            bar_full_char,
            bar_empty_char,
            bar_all_chars,
            fancy_bar_chars: ['<', '<', '>', '>'],
            header_dot: "·".to_string(),
            checkmark: "✓".to_string(),
            triangle: "▲".to_string(),
            full_circle: "●".to_string(),
            empty_circle: "○".to_string(),
            semicircle: "◐".to_string(),
            diamond: "◈".to_string(),
            empty_diamond: "◇".to_string(),
            arrow_up: "↑".to_string(),
            arrow_right: "→".to_string(),
            ellipsis: "…".to_string(),
            loading: "◌".to_string(),

            // Detail panel chars
            running: "●".to_string(),
            sleeping: "○".to_string(),
            disk_wait: "◐".to_string(),
            zombie: "✕".to_string(),
            stopped: "◼".to_string(),
            tracing: "◻".to_string(),
            idle: "◌".to_string(),
            unknown: "?".to_string(),

            app_icon: String::new(),
            ram_icon: String::new(),
            swap_icon: String::new(),
            process_list_icon: String::new(),
            insights_icon: String::new(),
            graph_icon: "◈".to_string(),
            memory_icon: String::new(),
            breakdown_icon: String::new(),

            border_subtle: Color::Rgb(0xd3, 0x96, 0x9b), // Not subtle, but I prefer it

            // Layout
            header_height: 1,
            center_height: 10,
            bottom_height: 4,
            left_width_percent: 40,
            side_vertical_split_percent: 60,

            invert_horizontal_split: false,
            invert_side_vertical_split: false,
            put_insights_on_top: false,
        }
    }

    /// Look a theme up by name. `custom` loads `custom_theme_file`, `hyprland`
    /// follows the current Hyprland/Omarchy desktop theme.
    pub fn from_name(name: &str, custom_theme_file: &str) -> Result<Self, String> {
        ThemeSource::from_name(name, custom_theme_file)?.load()
    }

    /// Theme that follows the current Hyprland desktop theme.
    ///
    /// Reads the theme's `colors.toml` (Omarchy's format, which also carries
    /// the Hyprland border colors). Search order:
    /// `$RAMWISE_HYPRLAND_THEME`, `$XDG_STATE_HOME/omarchy/current/theme/colors.toml`,
    /// `~/.local/state/omarchy/current/theme/colors.toml`,
    /// `~/.config/omarchy/current/theme/colors.toml`.
    pub fn hyprland() -> Result<Self, String> {
        ThemeSource::Hyprland {
            fallback_to_dark: false,
        }
        .load()
    }

    /// Build a theme from the contents of a Hyprland/Omarchy `colors.toml`
    pub fn from_hyprland_colors(contents: &str) -> Result<Self, String> {
        let palette = hyprland::Palette::parse(contents)?;
        Ok(palette.into_theme())
    }

    /// Load a theme from a TOML file. Missing keys fall back to the dark theme.
    pub fn custom(custom_theme_file: String) -> Result<Self, String> {
        ThemeSource::Custom(custom_theme_file).load()
    }

    /// Parse a theme from TOML. Missing keys fall back to the dark theme.
    pub fn from_toml(contents: &str) -> Result<Self, String> {
        let mut theme: Theme = toml::from_str(contents).map_err(|err| err.to_string())?;
        if theme.bar_partial_chars.is_empty() {
            return Err("bar_partial_chars must contain at least one character".to_string());
        }
        theme.rebuild_bar_all_chars();
        Ok(theme)
    }

    /// `icon` followed by a space, or nothing when the icon is empty
    pub fn icon(icon: &str) -> String {
        if icon.is_empty() {
            String::new()
        } else {
            format!("{icon} ")
        }
    }

    /// Switch every symbol to Nerd Font glyphs and use rounded progress bars.
    /// Needs a Nerd Font (Omarchy ships JetBrainsMono Nerd Font).
    pub fn use_nerd_font_symbols(&mut self) {
        let s = |v: &str| v.to_string();

        // Block bars (█░) colored as gradients. Partial steps (1/3 -> empty,
        // 2/3 -> full) quantise each cell, so a cell counts as filled once it
        // is at least half full.
        let empty = '░';
        let full = '█';
        self.bar_empty_char = empty.to_string();
        self.bar_full_char = full.to_string();
        self.bar_partial_chars = vec![empty, empty, full];
        self.rebuild_bar_all_chars();
        self.process_list_bar_type = BarType::Gradient;
        self.detail_panel_bar_type = BarType::Gradient;
        self.header_bar_type = BarType::Gradient;

        self.rank_top_symbol = s("\u{f01a5} "); // crown
        self.rank_high_symbol = s("\u{f04ce} "); // star
        self.chosen = s("\u{f0142}"); // chevron
        self.ignored_process_indicator = '\u{f009b}'; // bell off

        self.checkmark = s("\u{f05e0}"); // check circle
        self.triangle = s("\u{f0028}"); // alert circle (critical)
        self.full_circle = s("\u{f0026}"); // alert (warning)
        self.empty_circle = s("\u{f02fc}"); // information (info)
        self.semicircle = s("\u{f0431}"); // puzzle (fragmentation)
        self.diamond = s("\u{f08c6}"); // application
        self.empty_diamond = s("\u{f0a6}"); // pointing hand
        self.arrow_right = s("\u{f0054}");
        self.loading = s("\u{f051f}"); // hourglass

        self.running = s("\u{f040a}"); // play
        self.sleeping = s("\u{f04b2}"); // sleep
        self.disk_wait = s("\u{f02ca}"); // hard disk
        self.zombie = s("\u{f068c}"); // skull
        self.stopped = s("\u{f03e4}"); // pause
        self.tracing = s("\u{f00e4}"); // bug
        self.idle = s("\u{f0904}"); // power sleep
        self.unknown = s("\u{f02d7}"); // help circle

        self.app_icon = s("\u{f035b}"); // memory
        self.ram_icon = s("\u{f2db}"); // microchip
        self.swap_icon = s("\u{f04e1}"); // swap
        self.process_list_icon = s("\u{f0ae}"); // tasks
        self.insights_icon = s("\u{f0335}"); // lightbulb
        self.graph_icon = s("\u{f012a}"); // line chart
        self.memory_icon = s("\u{f035b}"); // memory
        self.breakdown_icon = s("\u{f200}"); // pie chart
    }

    /// Bar fill (0-100) for a process using `percent` of RAM: full at
    /// `process_list_scale_percent`, or at `highest_percent` (the biggest
    /// listed process) if that is larger
    pub fn process_list_bar_percent(&self, percent: f64, highest_percent: f64) -> f64 {
        let bar_scale = self
            .process_list_scale_percent
            .max(0.1)
            .max(highest_percent);
        (percent / bar_scale * 100.0).clamp(0.0, 100.0)
    }

    /// Memory color for a single process's share of RAM. A process rarely
    /// uses much of the system, so the steps are far lower than `mem_color`'s:
    /// critical from 15%, high from 12%, moderate from 9%, good from 5%.
    pub fn process_mem_color(&self, percent: f64) -> Color {
        match process_mem_level(percent) {
            4 => self.mem_critical,
            3 => self.mem_high,
            2 => self.mem_moderate,
            1 => self.mem_good,
            _ => self.mem_excellent,
        }
    }

    /// Dim variant of `process_mem_color` (end color of gradient bars)
    pub fn process_mem_color_dim(&self, percent: f64) -> Color {
        match process_mem_level(percent) {
            4 => self.mem_critical_dim,
            3 => self.mem_high_dim,
            2 => self.mem_moderate_dim,
            1 => self.mem_good_dim,
            _ => self.mem_excellent_dim,
        }
    }

    /// Recompute `bar_all_chars` (empty, partials, full) from the configured bar chars
    pub fn rebuild_bar_all_chars(&mut self) {
        let mut all = Vec::with_capacity(self.bar_partial_chars.len() + 2);
        if let Some(empty) = self.bar_empty_char.chars().next() {
            all.push(empty);
        }
        all.extend(self.bar_partial_chars.iter().copied());
        if let Some(full) = self.bar_full_char.chars().next() {
            all.push(full);
        }
        self.bar_all_chars = all;
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
    pub fn create_fancy_bar(&self, percent: f64, width: usize) -> String {
        let mut bar: Vec<char> = self.create_sleek_bar(percent, width).chars().collect();
        bar[0] = {
            let pos = self.bar_all_chars.iter().position(|&x| x == bar[0]);
            let fpos: f32 = if let Some(pos) = pos {
                pos as f32
            } else {
                -1.0
            };
            let len = self.bar_all_chars.len() as f32;
            let is_full = fpos / len;
            if is_full >= 0.5 {
                self.fancy_bar_chars[1]
            } else if is_full >= 0.0 {
                self.fancy_bar_chars[0]
            } else {
                'e'
            }
        };
        let last: usize = width - 1;
        bar[last] = {
            let pos = self.bar_all_chars.iter().position(|&x| x == bar[last]);
            let fpos: f32 = if let Some(pos) = pos {
                pos as f32
            } else {
                -1.0
            };
            let len = self.bar_all_chars.len() as f32;
            let is_full = fpos / len;
            if is_full >= 0.5 {
                self.fancy_bar_chars[3]
            } else if is_full >= 0.0 {
                self.fancy_bar_chars[2]
            } else {
                'e'
            }
        };
        let fancy_bar: String = bar.iter().collect();
        fancy_bar
    }
}
/// Color step for a process using `percent` of RAM: 0 (excellent) to 4 (critical)
fn process_mem_level(percent: f64) -> usize {
    const STEPS: [f64; 4] = [5.0, 9.0, 12.0, 15.0]; // good, moderate, high, critical
    STEPS.iter().filter(|&&step| percent >= step).count()
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

/// Hyprland / Omarchy desktop theme support
mod hyprland {
    use super::{Color, Theme};
    use std::collections::HashMap;
    use std::path::PathBuf;

    pub(super) fn candidate_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Ok(path) = std::env::var("RAMWISE_HYPRLAND_THEME")
            && !path.is_empty()
        {
            let expanded = shellexpand::tilde(&path).into_owned();
            paths.push(PathBuf::from(expanded));
        }
        let home = std::env::var("HOME").ok().filter(|h| !h.is_empty());
        let state_home = std::env::var("XDG_STATE_HOME")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|h| PathBuf::from(h).join(".local/state")));
        if let Some(state) = state_home {
            paths.push(state.join("omarchy/current/theme/colors.toml"));
        }
        if let Some(home) = &home {
            let fallback =
                PathBuf::from(home).join(".local/state/omarchy/current/theme/colors.toml");
            if !paths.contains(&fallback) {
                paths.push(fallback);
            }
            paths.push(PathBuf::from(home).join(".config/omarchy/current/theme/colors.toml"));
        }
        paths
    }

    /// Parse `#rrggbb`, `rgb(rrggbb)`, `rgba(rrggbbaa)` or `0xaarrggbb`
    pub(super) fn parse_color(value: &str) -> Option<Color> {
        let value = value.trim();
        let hex = if let Some(rest) = value.strip_prefix('#') {
            rest
        } else if let Some(rest) = value
            .strip_prefix("rgba(")
            .and_then(|r| r.strip_suffix(')'))
        {
            rest.get(..6)?
        } else if let Some(rest) = value.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
            rest
        } else {
            // 0xaarrggbb
            value.strip_prefix("0x")?.get(2..)?
        };
        if hex.len() != 6 {
            return None;
        }
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        Some(Color::Rgb(channel(0)?, channel(2)?, channel(4)?))
    }

    /// Scale an RGB color towards black
    pub(super) fn dim(color: Color, mult: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(
                (r as f32 * mult) as u8,
                (g as f32 * mult) as u8,
                (b as f32 * mult) as u8,
            ),
            other => other,
        }
    }

    pub(super) struct Palette {
        light: bool,
        colors: HashMap<String, Color>,
        /// First color of `hyprland_active_border`
        active_border: Option<Color>,
        /// First color of `hyprland_inactive_border`
        inactive_border: Option<Color>,
    }

    impl Palette {
        pub(super) fn parse(contents: &str) -> Result<Self, String> {
            let table: toml::Table = contents
                .parse()
                .map_err(|e: toml::de::Error| e.to_string())?;
            let mut colors = HashMap::new();
            for (key, value) in &table {
                if let Some(color) = value.as_str().and_then(parse_color) {
                    colors.insert(key.clone(), color);
                }
            }
            let first_border_color = |key: &str| {
                table
                    .get(key)
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.split_whitespace().find_map(parse_color))
            };
            let palette = Palette {
                light: table
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .is_some_and(|m| m.eq_ignore_ascii_case("light")),
                active_border: first_border_color("hyprland_active_border"),
                inactive_border: first_border_color("hyprland_inactive_border"),
                colors,
            };
            if palette.get(&["background", "color0"]).is_none()
                || palette.get(&["foreground", "color15", "color7"]).is_none()
            {
                return Err("colors.toml needs at least `background` and `foreground`".into());
            }
            Ok(palette)
        }

        /// First key that is present, checked in order
        fn get(&self, keys: &[&str]) -> Option<Color> {
            keys.iter().find_map(|k| self.colors.get(*k).copied())
        }

        fn pick(&self, keys: &[&str], fallback: Color) -> Color {
            self.get(keys).unwrap_or(fallback)
        }

        pub(super) fn into_theme(self) -> Theme {
            // Symbols, bar styles and layout come from the matching built-in theme
            let mut theme = if self.light {
                Theme::light()
            } else {
                Theme::dark()
            };
            let dim_mult = if self.light { 0.9 } else { 0.75 };

            let bg = self.pick(&["background", "color0"], theme.bg);
            let fg = self.pick(&["foreground", "color15", "color7"], theme.fg);
            let accent = self.pick(&["accent", "blue", "color4"], theme.primary);
            let red = self.pick(&["red", "color1"], theme.error);
            let green = self.pick(&["green", "color2"], theme.success);
            let yellow = self.pick(&["yellow", "color3"], theme.warning);
            let blue = self.pick(&["blue", "color4"], theme.info);
            let magenta = self.pick(&["magenta", "color5"], theme.tertiary);
            let cyan = self.pick(&["cyan", "color6"], theme.secondary);
            let orange = self.pick(&["orange", "bright_red", "color9"], red);
            let elevated = self.pick(&["lighter_background", "selection", "color8"], bg);
            let selection_bg = self.pick(
                &["selection_background", "selection", "lighter_background"],
                accent,
            );

            theme.bg = bg;
            theme.fg = fg;
            theme.fg_dim = self.pick(&["light_foreground", "color7"], fg);
            theme.fg_muted = self.pick(&["dark_foreground", "muted", "color8"], theme.fg_dim);

            theme.primary = accent;
            theme.primary_dim = dim(accent, dim_mult);
            theme.secondary = cyan;
            theme.tertiary = magenta;

            theme.error = red;
            theme.warning = yellow;
            theme.success = green;
            theme.info = blue;

            theme.mem_excellent = green;
            theme.mem_good = self.pick(&["bright_green", "color10"], green);
            theme.mem_moderate = yellow;
            theme.mem_high = orange;
            theme.mem_critical = red;
            theme.mem_excellent_dim = dim(theme.mem_excellent, dim_mult);
            theme.mem_good_dim = dim(theme.mem_good, dim_mult);
            theme.mem_moderate_dim = dim(theme.mem_moderate, dim_mult);
            theme.mem_high_dim = dim(theme.mem_high, dim_mult);
            theme.mem_critical_dim = dim(theme.mem_critical, dim_mult);

            theme.border = self.inactive_border.unwrap_or(elevated);
            theme.border_focused = self.active_border.unwrap_or(accent);
            theme.border_subtle = red;
            theme.selection_bg = selection_bg;
            theme.selection_fg = self.pick(&["selection_foreground", "bright_foreground"], fg);
            theme.header_bg = elevated;

            theme.graph_line = accent;
            theme.graph_border = theme.border;

            theme.rank_top = yellow;
            theme.rank_high = accent;

            // Omarchy ships a Nerd Font, so use its icons and rounded bars
            theme.use_nerd_font_symbols();
            theme
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn to_toml_value(theme: &Theme) -> toml::Value {
        toml::Value::try_from(theme).expect("theme serializes")
    }

    #[test]
    fn example_theme_file_lists_every_key_with_dark_defaults() {
        let file = include_str!("../../theme.toml");
        let file_keys: std::collections::BTreeSet<String> = file
            .parse::<toml::Table>()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        let dark = to_toml_value(&Theme::dark());
        let theme_keys: std::collections::BTreeSet<String> =
            dark.as_table().unwrap().keys().cloned().collect();
        assert_eq!(file_keys, theme_keys);

        let parsed = Theme::from_toml(file).expect("theme.toml parses");
        assert_eq!(to_toml_value(&parsed), dark);
        assert_eq!(parsed.bar_all_chars, Theme::dark().bar_all_chars);
    }

    #[test]
    fn built_in_themes_round_trip_through_toml() {
        for theme in [Theme::dark(), Theme::light(), Theme::ansi4bit()] {
            let text = toml::to_string(&theme).unwrap();
            let parsed = Theme::from_toml(&text).unwrap();
            assert_eq!(to_toml_value(&parsed), to_toml_value(&theme));
            assert_eq!(parsed.bar_all_chars, theme.bar_all_chars);
        }
    }

    #[test]
    fn custom_theme_accepts_every_color_form_and_defaults_the_rest() {
        let theme = Theme::from_toml(
            r##"
            bg = [1, 2, 3]
            fg = "#0a0b0c"
            primary = "lightblue"
            secondary = 12
            graph_marker = "half_block"
            header_bar_type = "fancy_gradient"
            swap_display = "icon"
            bar_empty_char = "-"
            ignored_process_indicator = "x"
            "##,
        )
        .unwrap();
        assert_eq!(theme.bg, Color::Rgb(1, 2, 3));
        assert_eq!(theme.fg, Color::Rgb(10, 11, 12));
        assert_eq!(theme.primary, Color::LightBlue);
        assert_eq!(theme.secondary, Color::Indexed(12));
        assert_eq!(theme.graph_marker, Marker::HalfBlock);
        assert_eq!(theme.header_bar_type, BarType::FancyGradient);
        assert_eq!(theme.swap_display, SwapDisplay::Icon);
        assert_eq!(theme.ignored_process_indicator, 'x');
        assert_eq!(theme.bar_all_chars.first(), Some(&'-'));
        // Untouched keys keep the dark theme's values
        assert_eq!(theme.error, Theme::dark().error);
        assert_eq!(theme.header_height, Theme::dark().header_height);
    }

    #[test]
    fn custom_theme_rejects_bad_values() {
        assert!(Theme::from_toml("bg = \"not-a-color\"").is_err());
        assert!(Theme::from_toml("graph_marker = \"zigzag\"").is_err());
        assert!(Theme::from_toml("bar_partial_chars = []").is_err());
    }

    #[test]
    fn graph_marker_accepts_single_character() {
        let theme = Theme::from_toml("graph_marker = \"*\"").unwrap();
        assert_eq!(theme.graph_marker, Marker::Custom('*'));
    }

    #[test]
    fn from_name_knows_every_theme() {
        assert!(Theme::from_name("dark", "").is_ok());
        assert!(Theme::from_name(" Light ", "").is_ok());
        assert!(Theme::from_name("ansi", "").is_ok());
        assert!(Theme::from_name("unknown-theme", "").is_err());
        assert!(Theme::from_name("custom", "/nonexistent/ramwise-theme.toml").is_err());
    }

    const OMARCHY_COLORS: &str = r##"
        mode = "dark"
        accent = "#8fbf8f"
        selection = "#3a4540"
        selection_background = "#3a4540"
        selection_foreground = "#eceff0"
        muted = "#5f676c"
        background = "#1f2326"
        lighter_background = "#2c3135"
        foreground = "#d4d9d6"
        dark_foreground = "#7d858a"
        light_foreground = "#b3bab8"
        red = "#c97064"
        yellow = "#c9b77a"
        orange = "#c4906a"
        green = "#8fbf8f"
        cyan = "#84b5ad"
        blue = "#8aa4b8"
        magenta = "#a99ab8"
        bright_green = "#aedaae"
        hyprland_active_border = "rgb(aedaae) rgb(6b757b) 45deg"
        hyprland_inactive_border = "rgba(3a4045aa)"
    "##;

    #[test]
    fn hyprland_theme_maps_omarchy_colors() {
        let theme = Theme::from_hyprland_colors(OMARCHY_COLORS).unwrap();
        assert_eq!(theme.bg, Color::Rgb(0x1f, 0x23, 0x26));
        assert_eq!(theme.fg, Color::Rgb(0xd4, 0xd9, 0xd6));
        assert_eq!(theme.primary, Color::Rgb(0x8f, 0xbf, 0x8f));
        assert_eq!(theme.secondary, Color::Rgb(0x84, 0xb5, 0xad));
        assert_eq!(theme.error, Color::Rgb(0xc9, 0x70, 0x64));
        assert_eq!(theme.mem_high, Color::Rgb(0xc4, 0x90, 0x6a));
        assert_eq!(theme.mem_good, Color::Rgb(0xae, 0xda, 0xae));
        assert_eq!(theme.border_focused, Color::Rgb(0xae, 0xda, 0xae));
        assert_eq!(theme.border, Color::Rgb(0x3a, 0x40, 0x45));
        assert_eq!(theme.selection_fg, Color::Rgb(0xec, 0xef, 0xf0));
        assert_eq!(theme.header_bg, Color::Rgb(0x2c, 0x31, 0x35));
        // Omarchy ships a Nerd Font: rounded progress bars and icons
        assert_eq!(theme.header_bar_type, BarType::Gradient);
        assert_eq!(theme.bar_full_char, "█");
        assert_eq!(theme.app_icon, "\u{f035b}");
    }

    #[test]
    fn hyprland_theme_falls_back_when_keys_are_missing() {
        // No accent, orange, borders or selection keys; numbered palette only
        let theme = Theme::from_hyprland_colors(
            r##"
            mode = "light"
            background = "#ffffff"
            foreground = "#000000"
            color1 = "#aa0000"
            color4 = "#0000aa"
            "##,
        )
        .unwrap();
        assert_eq!(theme.primary, Color::Rgb(0, 0, 0xaa));
        assert_eq!(theme.error, Color::Rgb(0xaa, 0, 0));
        assert_eq!(theme.mem_high, Color::Rgb(0xaa, 0, 0));
        assert_eq!(theme.border_focused, Color::Rgb(0, 0, 0xaa));
        // Light mode starts from the light theme (no green given, so the
        // memory gradient starts at the light theme's success color)
        assert_eq!(theme.mem_excellent, Theme::light().success);
    }

    #[test]
    fn hyprland_theme_needs_background_and_foreground() {
        assert!(Theme::from_hyprland_colors("accent = \"#ffffff\"").is_err());
        assert!(Theme::from_hyprland_colors("not toml at all ===").is_err());
    }

    #[test]
    fn nerd_font_bars_are_quantised_to_whole_cells() {
        let mut theme = Theme::dark();
        theme.use_nerd_font_symbols();
        let (empty, full) = ('░', '█');
        let bar = |percent: f64| theme.create_sleek_bar(percent, 10);
        assert_eq!(bar(0.0), empty.to_string().repeat(10));
        assert_eq!(bar(100.0), full.to_string().repeat(10));
        // 34% of 10 cells: 3.4 rounds down, 36%: 3.6 rounds up
        assert_eq!(
            bar(34.0),
            format!(
                "{}{}",
                full.to_string().repeat(3),
                empty.to_string().repeat(7)
            )
        );
        assert_eq!(
            bar(36.0),
            format!(
                "{}{}",
                full.to_string().repeat(4),
                empty.to_string().repeat(6)
            )
        );
    }

    #[test]
    fn process_list_scale() {
        let theme = Theme::dark();
        // Colors: critical from 15% of RAM
        assert_eq!(theme.process_mem_color(15.0), theme.mem_critical);
        assert_eq!(theme.process_mem_color(40.0), theme.mem_critical);
        assert_eq!(theme.process_mem_color(12.0), theme.mem_high);
        assert_eq!(theme.process_mem_color(9.0), theme.mem_moderate);
        assert_eq!(theme.process_mem_color(5.0), theme.mem_good);
        assert_eq!(theme.process_mem_color(4.9), theme.mem_excellent);
        assert_eq!(theme.process_mem_color_dim(15.0), theme.mem_critical_dim);

        // Bars: full at 50%, or at the biggest process if that uses more
        assert_eq!(theme.process_list_bar_percent(25.0, 30.0), 50.0);
        assert_eq!(theme.process_list_bar_percent(50.0, 50.0), 100.0);
        assert_eq!(theme.process_list_bar_percent(20.0, 80.0), 25.0);
    }

    #[test]
    fn hyprland_color_formats() {
        use super::hyprland::parse_color;
        let expected = Some(Color::Rgb(0x12, 0x34, 0x56));
        assert_eq!(parse_color("#123456"), expected);
        assert_eq!(parse_color("rgb(123456)"), expected);
        assert_eq!(parse_color("rgba(123456ee)"), expected);
        assert_eq!(parse_color("0xee123456"), expected);
        assert_eq!(parse_color("45deg"), None);
        assert_eq!(parse_color("#12345"), None);
    }

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
        let empty_chars: f64 = chars as f64 * (1.0 - part / 100.0);
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
        let empty_chars: f64 = chars as f64 * (1.0 - part / 100.0);
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
        let empty_chars: f64 = chars as f64 * (1.0 - part / 100.0);
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
        let empty_chars: f64 = chars as f64 * (1.0 - part / 100.0);
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
        let empty_chars: f64 = chars as f64 * (1.0 - part / 100.0);
        let incorrect_result_a = theme.bar_full_char.to_string().repeat(full_chars as usize)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize);
        print!("{}", incorrect_result_a);
        let incorrect_result_b = theme
            .bar_full_char
            .to_string()
            .repeat(full_chars as usize + 1)
            + &theme
                .bar_empty_char
                .to_string()
                .repeat(empty_chars as usize - 1);
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
        let empty_chars: f64 = chars as f64 * (1.0 - part / 100.0);
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
        let empty_chars: f64 = chars as f64 * (1.0 - part / 100.0);
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
