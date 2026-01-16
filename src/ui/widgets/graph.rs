//! Modern memory trend graph widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Widget},
};

use crate::history::HistoryBuffer;
use crate::ui::Theme;
use crate::utils::format_bytes;

/// Modern graph widget for memory trends
pub struct GraphWidget<'a> {
    history: &'a HistoryBuffer,
    selected_pid: Option<i32>,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> GraphWidget<'a> {
    pub fn new(history: &'a HistoryBuffer, theme: &'a Theme) -> Self {
        Self {
            history,
            selected_pid: None,
            theme,
            focused: false,
        }
    }

    pub fn selected_pid(mut self, pid: Option<i32>) -> Self {
        self.selected_pid = pid;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<'a> Widget for GraphWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Determine what to graph
        let (title, data, y_bounds, y_labels, is_process) = if let Some(pid) = self.selected_pid {
            let trend = self.history.process_trend(pid);
            if trend.is_empty() {
                render_empty(area, buf, self.theme, self.focused, "Collecting data…");
                return;
            }

            // Convert to chart data points (x = time index, y = bytes)
            let data: Vec<(f64, f64)> = trend
                .iter()
                .enumerate()
                .map(|(i, (_, bytes))| (i as f64, *bytes as f64))
                .collect();

            let max_bytes = trend.iter().map(|(_, b)| *b).max().unwrap_or(1) as f64;
            let min_bytes = trend.iter().map(|(_, b)| *b).min().unwrap_or(0) as f64;

            // Add 10% padding
            let range = (max_bytes - min_bytes).max(1.0);
            let y_min = (min_bytes - range * 0.1).max(0.0);
            let y_max = max_bytes + range * 0.1;

            let y_labels = vec![
                Span::styled(format_bytes(y_min as u64), Style::default().fg(self.theme.fg_muted)),
                Span::styled(format_bytes(((y_min + y_max) / 2.0) as u64), Style::default().fg(self.theme.fg_dim)),
                Span::styled(format_bytes(y_max as u64), Style::default().fg(self.theme.fg_muted)),
            ];

            ("Process Memory", data, [y_min, y_max], y_labels, true)
        } else {
            // System memory trend
            let trend = self.history.system_trend();
            if trend.is_empty() {
                render_empty(area, buf, self.theme, self.focused, "Collecting data…");
                return;
            }

            let data: Vec<(f64, f64)> = trend
                .iter()
                .enumerate()
                .map(|(i, (_, bytes))| (i as f64, *bytes as f64))
                .collect();

            let max_bytes = trend.iter().map(|(_, b)| *b).max().unwrap_or(1) as f64;
            let min_bytes = trend.iter().map(|(_, b)| *b).min().unwrap_or(0) as f64;

            let range = (max_bytes - min_bytes).max(1.0);
            let y_min = (min_bytes - range * 0.1).max(0.0);
            let y_max = max_bytes + range * 0.1;

            let y_labels = vec![
                Span::styled(format_bytes(y_min as u64), Style::default().fg(self.theme.fg_muted)),
                Span::styled(format_bytes(((y_min + y_max) / 2.0) as u64), Style::default().fg(self.theme.fg_dim)),
                Span::styled(format_bytes(y_max as u64), Style::default().fg(self.theme.fg_muted)),
            ];

            ("System Memory", data, [y_min, y_max], y_labels, false)
        };

        if data.is_empty() {
            render_empty(area, buf, self.theme, self.focused, "No data");
            return;
        }

        let x_max = data.len().saturating_sub(1) as f64;
        let time_span = data.len();

        // Choose line color based on context
        let line_color = if is_process {
            self.theme.primary
        } else {
            self.theme.secondary
        };

        let datasets = vec![
            Dataset::default()
                .name("Memory")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(line_color))
                .data(&data),
        ];

        // Modern time labels
        let x_labels = vec![
            Span::styled("now", Style::default().fg(self.theme.fg_dim)),
            Span::styled(format!("-{}s", time_span), Style::default().fg(self.theme.fg_muted)),
        ];

        // Modern title
        let title_line = Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("◈ ", Style::default().fg(line_color)),
            Span::styled(title, Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled(" ", Style::default()),
        ]);

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(title_line)
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style(self.focused))
                    .style(Style::default().bg(self.theme.bg)),
            )
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(self.theme.border_subtle))
                    .bounds([0.0, x_max])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(self.theme.border_subtle))
                    .bounds(y_bounds)
                    .labels(y_labels),
            );

        chart.render(area, buf);
    }
}

fn render_empty(area: Rect, buf: &mut Buffer, theme: &Theme, focused: bool, message: &str) {
    let title = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("◈ ", Style::default().fg(theme.fg_muted)),
        Span::styled("Memory Trend", Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default()),
    ]);
    
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.border_style(focused))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    block.render(area, buf);

    // Center the message with loading indicator
    let display = format!("◌ {}", message);
    let x = inner.x + (inner.width.saturating_sub(display.len() as u16)) / 2;
    let y = inner.y + inner.height / 2;

    buf.set_string(x, y, display, theme.muted_style());
}
