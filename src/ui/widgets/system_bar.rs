//! System memory bar widget (compact overview)

#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};

use crate::collector::SystemMemory;
use crate::ui::Theme;
use crate::utils::format_bytes;

/// Compact system memory bar
pub struct SystemBarWidget<'a> {
    system: &'a SystemMemory,
    theme: &'a Theme,
}

impl<'a> SystemBarWidget<'a> {
    pub fn new(system: &'a SystemMemory, theme: &'a Theme) -> Self {
        Self { system, theme }
    }
}

impl<'a> Widget for SystemBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 1 {
            return;
        }

        let sys = self.system;
        let total = sys.total as f64;

        if total == 0.0 {
            return;
        }

        // Calculate percentages
        let used_pct = sys.used() as f64 / total;
        let cached_pct = sys.cached as f64 / total;
        let buffers_pct = sys.buffers as f64 / total;
        let _free_pct = sys.free as f64 / total;

        // Calculate bar segments (leave space for labels)
        let label_width = 30u16;
        let bar_width = area.width.saturating_sub(label_width) as usize;

        // Segment widths
        let used_width = (used_pct * bar_width as f64).round() as usize;
        let cached_width = (cached_pct * bar_width as f64).round() as usize;
        let buffers_width = (buffers_pct * bar_width as f64).round() as usize;
        let free_width = bar_width.saturating_sub(used_width + cached_width + buffers_width);

        // Build the bar
        let mut x = area.x;
        let y = area.y;

        // Used segment (colored by percentage)
        let used_color = self.theme.mem_color(used_pct * 100.0);
        for _ in 0..used_width.min(bar_width) {
            buf.set_string(x, y, "█", Style::default().fg(used_color));
            x += 1;
        }

        // Cached segment
        for _ in 0..cached_width {
            if (x - area.x) as usize >= bar_width {
                break;
            }
            buf.set_string(x, y, "▓", Style::default().fg(self.theme.info));
            x += 1;
        }

        // Buffers segment
        for _ in 0..buffers_width {
            if (x - area.x) as usize >= bar_width {
                break;
            }
            buf.set_string(x, y, "▒", Style::default().fg(self.theme.secondary_dim));
            x += 1;
        }

        // Free segment
        for _ in 0..free_width {
            if (x - area.x) as usize >= bar_width {
                break;
            }
            buf.set_string(x, y, "░", Style::default().fg(self.theme.fg_dim));
            x += 1;
        }

        // Legend
        let legend = format!(
            " Used:{} Cache:{} Free:{}",
            format_bytes(sys.used()),
            format_bytes(sys.cached),
            format_bytes(sys.free)
        );

        let legend_x = area.x + bar_width as u16 + 1;
        if legend_x + legend.len() as u16 <= area.x + area.width {
            buf.set_string(legend_x, y, &legend, self.theme.dim_style());
        }
    }
}
