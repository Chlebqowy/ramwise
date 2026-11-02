//! Modern header widget with sleek design

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::collector::SystemMemory;
use crate::ui::Theme;
use crate::utils::format_bytes;

use crate::ui::theme::{BarType, SwapDisplay};

/// Modern header bar widget
pub struct HeaderWidget<'a> {
    system: &'a SystemMemory,
    theme: &'a Theme,
    version: &'static str,
}

impl<'a> HeaderWidget<'a> {
    pub fn new(system: &'a SystemMemory, theme: &'a Theme) -> Self {
        Self {
            system,
            theme,
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

impl<'a> Widget for HeaderWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const RAM_BAR_WIDTH: usize = 12;
        const SWAP_BAR_WIDTH: usize = 8;
        let sys = self.system;
        let mut ram = vec![];
        ram.push(Span::styled("RAM ", Style::default().fg(self.theme.fg_dim)));

        // Build the header line with modern styling

        // App title with icon
        let title = Span::styled(
            format!(" ramwise v{}", self.version),
            Style::default()
                .fg(self.theme.primary)
                .add_modifier(Modifier::BOLD),
        );

        let dot = Span::styled(
            format!(" { } ", self.theme.header_dot),
            self.theme.muted_style(),
        );

        // RAM usage with truly smooth gradient bar
        let ram_percent = sys.usage_percent();
        let ram_color = self.theme.mem_color(ram_percent);
        push_bar(self.theme, ram_percent, RAM_BAR_WIDTH, &mut ram);

        ram.push(Span::styled(
            format!(" {}/{} ", format_bytes(sys.used()), format_bytes(sys.total),),
            Style::default().fg(self.theme.fg),
        ));
        ram.push(Span::styled(
            format!("{:.0}%", ram_percent),
            Style::default().fg(ram_color).add_modifier(Modifier::BOLD),
        ));

        // Swap usage, either as a bar or as a status icon
        let swap_percent = sys.swap_percent();
        let mut swap = vec![];
        if sys.swap_total > 0 {
            let swap_color = match self.theme.swap_display {
                SwapDisplay::Bar => self.theme.mem_color(swap_percent),
                SwapDisplay::Icon if swap_percent > 80.0 => self.theme.error,
                SwapDisplay::Icon if swap_percent > 50.0 => self.theme.warning,
                SwapDisplay::Icon => self.theme.fg_dim,
            };
            swap.push(dot.clone());
            match self.theme.swap_display {
                SwapDisplay::Bar => {
                    swap.push(Span::styled(
                        "Swap ",
                        Style::default().fg(self.theme.fg_dim),
                    ));
                    push_bar(self.theme, swap_percent, SWAP_BAR_WIDTH, &mut swap);
                }
                SwapDisplay::Icon => {
                    let status_icon = if swap_percent > 80.0 {
                        self.theme.triangle.clone()
                    } else if swap_percent > 50.0 {
                        self.theme.full_circle.clone()
                    } else {
                        self.theme.empty_circle.clone()
                    };
                    swap.push(Span::styled(
                        "Swap ",
                        Style::default().fg(self.theme.fg_dim),
                    ));
                    swap.push(Span::styled(status_icon, Style::default().fg(swap_color)));
                }
            }
            swap.push(Span::styled(
                format!(
                    " {}/{} ",
                    format_bytes(sys.swap_used),
                    format_bytes(sys.swap_total),
                ),
                Style::default().fg(self.theme.fg),
            ));
            swap.push(Span::styled(
                format!("{:.0}%", swap_percent),
                Style::default().fg(swap_color).add_modifier(Modifier::BOLD),
            ));
        }

        // Help hint with modern styling (right-aligned)
        let help = vec![
            Span::styled(
                "?",
                Style::default()
                    .fg(self.theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Help  ", self.theme.muted_style()),
            Span::styled(
                "q",
                Style::default()
                    .fg(self.theme.tertiary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Quit ", self.theme.muted_style()),
        ];

        // Combine all parts
        let mut spans = vec![title, dot.clone()];
        spans.extend(ram);
        spans.extend(swap);

        // Calculate padding for right alignment
        let content_width: usize = spans.iter().map(|s| s.width()).sum();
        let help_width: usize = help.iter().map(|s| s.width()).sum();
        let padding = (area.width as usize).saturating_sub(content_width + help_width);
        if padding > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
        }
        spans.extend(help);

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).style(Style::default().bg(self.theme.header_bg));

        paragraph.render(area, buf);
    }
}

/// Render a usage bar into `spans` following the theme's `header_bar_type`
fn push_bar<'a>(theme: &Theme, percent: f64, width: usize, spans: &mut Vec<Span<'a>>) {
    let color = theme.mem_color(percent);
    let bar = match theme.header_bar_type {
        BarType::Standard | BarType::Gradient => theme.create_sleek_bar(percent, width),
        BarType::Fancy | BarType::FancyGradient => theme.create_fancy_bar(percent, width),
    };
    if matches!(
        theme.header_bar_type,
        BarType::Gradient | BarType::FancyGradient
    ) {
        Theme::create_gradient_bar(bar, width, color, theme.mem_color_dim(percent), spans);
    } else {
        spans.push(Span::styled(bar, Style::default().fg(color)));
    }
}
