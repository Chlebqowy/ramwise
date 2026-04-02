use serde::Deserialize;
use std::fs;

use ui::Layout;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub layout: Layout,
}
impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: Layout::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
/// Main layout manager
pub struct Layout {
    /// Header height
    pub header_height: u16,
    /// Bottom panel height
    pub bottom_height: u16,
    /// Left panel width percentage
    pub left_width_percent: u16,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            header_height: 1,
            bottom_height: 4,
            left_width_percent: 40,
        }
    }
}