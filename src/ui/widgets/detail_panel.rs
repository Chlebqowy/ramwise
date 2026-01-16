//! Modern detail panel widget with card-like sections

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::collector::ProcessMemory;
use crate::ui::Theme;
use crate::utils::format_bytes;

/// Modern detail panel widget
pub struct DetailPanelWidget<'a> {
    process: Option<&'a ProcessMemory>,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> DetailPanelWidget<'a> {
    pub fn new(process: Option<&'a ProcessMemory>, theme: &'a Theme) -> Self {
        Self {
            process,
            theme,
            focused: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<'a> Widget for DetailPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Modern title with icon-like prefix
        let title = match &self.process {
            Some(p) => Line::from(vec![
                Span::styled(" ◆ ", Style::default().fg(self.theme.primary)),
                Span::styled(&p.name, Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD)),
                Span::styled(" · ", self.theme.muted_style()),
                Span::styled(format!("PID {}", p.pid), Style::default().fg(self.theme.secondary)),
                Span::styled(" ", Style::default()),
            ]),
            None => Line::from(vec![
                Span::styled(" ◇ ", self.theme.muted_style()),
                Span::styled("Select a process", self.theme.muted_style()),
                Span::styled(" ", Style::default()),
            ]),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(self.focused))
            .style(Style::default().bg(self.theme.bg));

        let inner = block.inner(area);
        block.render(area, buf);

        let Some(proc) = self.process else {
            // Render placeholder with icon
            let lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("   ↑ ", self.theme.muted_style()),
                    Span::styled("Use ", self.theme.dim_style()),
                    Span::styled("j/k", Style::default().fg(self.theme.secondary)),
                    Span::styled(" or ", self.theme.dim_style()),
                    Span::styled("↑/↓", Style::default().fg(self.theme.secondary)),
                    Span::styled(" to navigate", self.theme.dim_style()),
                ]),
            ];
            let placeholder = Paragraph::new(lines);
            placeholder.render(inner, buf);
            return;
        };

        // Build content with modern sections
        let mut lines = Vec::new();

        // Process info row with status chip
        let state_style = match proc.state {
            'R' => Style::default().fg(self.theme.success),
            'S' => Style::default().fg(self.theme.info),
            'D' => Style::default().fg(self.theme.warning),
            'Z' => Style::default().fg(self.theme.error),
            _ => Style::default().fg(self.theme.fg_dim),
        };
        
        lines.push(Line::from(vec![
            Span::styled(state_chip(proc.state), state_style),
            Span::styled("  ", Style::default()),
            Span::styled("PPID ", self.theme.muted_style()),
            Span::styled(proc.ppid.to_string(), Style::default().fg(self.theme.fg_dim)),
            Span::styled("  UID ", self.theme.muted_style()),
            Span::styled(proc.uid.to_string(), Style::default().fg(self.theme.fg_dim)),
        ]));

        // Command line (styled as code)
        let max_cmd_len = inner.width as usize - 2;
        let cmdline = if proc.cmdline.len() > max_cmd_len {
            format!("{}…", &proc.cmdline[..max_cmd_len.saturating_sub(1)])
        } else {
            proc.cmdline.clone()
        };
        lines.push(Line::from(vec![
            Span::styled(cmdline, Style::default().fg(self.theme.fg_muted)),
        ]));

        lines.push(Line::from(""));

        // Memory section header
        lines.push(section_header("Memory", self.theme));

        // Main memory stats with visual bars
        let total_for_bar = proc.vss.max(proc.rss);
        
        lines.push(create_memory_row(
            "RSS",
            proc.rss,
            total_for_bar,
            "Resident memory",
            self.theme,
            true,
        ));
        
        lines.push(create_memory_row(
            "VSS",
            proc.vss,
            total_for_bar,
            "Virtual size",
            self.theme,
            false,
        ));

        if proc.pss > 0 {
            lines.push(create_memory_row(
                "PSS",
                proc.pss,
                proc.rss,
                "Proportional",
                self.theme,
                false,
            ));
        }

        if proc.uss > 0 {
            lines.push(create_memory_row(
                "USS",
                proc.uss,
                proc.rss,
                "Unique",
                self.theme,
                false,
            ));
        }

        lines.push(Line::from(""));

        // Breakdown section
        lines.push(section_header("Breakdown", self.theme));

        // Two-column layout for breakdown
        lines.push(create_two_col(
            ("Shared", proc.shared),
            ("Private", proc.private),
            self.theme,
        ));
        
        lines.push(create_two_col(
            ("Heap", proc.heap),
            ("Stack", proc.stack),
            self.theme,
        ));

        lines.push(create_two_col(
            ("Libraries", proc.libs),
            ("Anon", proc.anonymous),
            self.theme,
        ));

        // Swap indicator (if any)
        if proc.swap > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("▲ ", Style::default().fg(self.theme.warning)),
                Span::styled("Swap: ", self.theme.dim_style()),
                Span::styled(
                    format_bytes(proc.swap),
                    self.theme.warning_style(),
                ),
            ]));
        }

        // Page faults with severity indication
        if proc.major_faults > 100 {
            lines.push(Line::from(vec![
                Span::styled("● ", Style::default().fg(if proc.major_faults > 1000 { self.theme.warning } else { self.theme.fg_muted })),
                Span::styled("Page faults: ", self.theme.dim_style()),
                Span::styled(
                    format!("{} major", proc.major_faults),
                    if proc.major_faults > 1000 {
                        self.theme.warning_style()
                    } else {
                        Style::default().fg(self.theme.fg_dim)
                    },
                ),
            ]));
        }

        // Fragmentation indicator
        let frag_ratio = proc.fragmentation_ratio();
        if frag_ratio > 5.0 {
            lines.push(Line::from(vec![
                Span::styled("◐ ", Style::default().fg(if frag_ratio > 15.0 { self.theme.warning } else { self.theme.info })),
                Span::styled("Fragmentation: ", self.theme.dim_style()),
                Span::styled(
                    format!("{:.0}x", frag_ratio),
                    if frag_ratio > 15.0 {
                        self.theme.warning_style()
                    } else {
                        Style::default().fg(self.theme.fg_dim)
                    },
                ),
                Span::styled(" VSS/RSS ratio", self.theme.muted_style()),
            ]));
        }

        let paragraph = Paragraph::new(lines);
        paragraph.render(inner, buf);
    }
}

/// Create a section header with modern styling
fn section_header<'a>(label: &'a str, theme: &'a Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
    ])
}

/// Create a memory row with mini bar
fn create_memory_row<'a>(
    label: &'a str,
    value: u64,
    max_value: u64,
    _desc: &'a str,
    theme: &'a Theme,
    is_primary: bool,
) -> Line<'a> {
    let bar_width = 12;
    let percent = if max_value > 0 {
        (value as f64 / max_value as f64) * 100.0
    } else {
        0.0
    };
    
    let bar = create_mini_bar(percent.min(100.0), bar_width);
    let color = if is_primary {
        theme.primary
    } else {
        theme.fg_dim
    };
    
    Line::from(vec![
        Span::styled(format!("{:<4}", label), Style::default().fg(color)),
        Span::styled(bar, Style::default().fg(color)),
        Span::raw(" "),
        Span::styled(
            format!("{:>9}", format_bytes(value)),
            if is_primary {
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg_dim)
            },
        ),
    ])
}

/// Create a two-column stat row
fn create_two_col<'a>(
    left: (&'a str, u64),
    right: (&'a str, u64),
    theme: &'a Theme,
) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<9}", left.0), theme.muted_style()),
        Span::styled(format!("{:>9}", format_bytes(left.1)), Style::default().fg(theme.fg_dim)),
        Span::styled("   ", Style::default()),
        Span::styled(format!("{:<9}", right.0), theme.muted_style()),
        Span::styled(format!("{:>9}", format_bytes(right.1)), Style::default().fg(theme.fg_dim)),
    ])
}

/// Create a mini progress bar
fn create_mini_bar(percent: f64, width: usize) -> String {
    let chars = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    let total_eighths = ((percent / 100.0) * (width * 8) as f64).round() as usize;
    let full_blocks = total_eighths / 8;
    let partial = total_eighths % 8;
    
    let mut bar = "█".repeat(full_blocks.min(width));
    
    if partial > 0 && bar.chars().count() < width {
        bar.push(chars[partial]);
    }
    
    let remaining = width.saturating_sub(bar.chars().count());
    bar.push_str(&"░".repeat(remaining));
    
    bar
}

/// State chip with icon
fn state_chip(state: char) -> String {
    match state {
        'R' => "● Running".to_string(),
        'S' => "○ Sleeping".to_string(),
        'D' => "◐ Disk Wait".to_string(),
        'Z' => "✕ Zombie".to_string(),
        'T' => "◼ Stopped".to_string(),
        't' => "◻ Tracing".to_string(),
        'I' => "◌ Idle".to_string(),
        _ => format!("? {}", state),
    }
}
