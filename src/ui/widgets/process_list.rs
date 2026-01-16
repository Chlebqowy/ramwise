//! Modern process list widget with clean design

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::collector::ProcessMemory;
use crate::ui::Theme;
use crate::utils::format_bytes;

/// Sorting mode for process list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Rss,
    Pss,
    Private,
    Name,
    Pid,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Rss => "RSS",
            SortMode::Pss => "PSS",
            SortMode::Private => "Private",
            SortMode::Name => "Name",
            SortMode::Pid => "PID",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SortMode::Rss => SortMode::Pss,
            SortMode::Pss => SortMode::Private,
            SortMode::Private => SortMode::Name,
            SortMode::Name => SortMode::Pid,
            SortMode::Pid => SortMode::Rss,
        }
    }
}

/// Process list widget state
pub struct ProcessListState {
    pub list_state: ListState,
    pub sort_mode: SortMode,
    pub selected_pid: Option<i32>,
}

impl ProcessListState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            list_state: state,
            sort_mode: SortMode::Rss,
            selected_pid: None,
        }
    }

    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
    }
}

impl Default for ProcessListState {
    fn default() -> Self {
        Self::new()
    }
}

/// Modern process list widget
pub struct ProcessListWidget<'a> {
    processes: &'a [ProcessMemory],
    theme: &'a Theme,
    focused: bool,
    total_memory: u64,
}

impl<'a> ProcessListWidget<'a> {
    pub fn new(
        processes: &'a [ProcessMemory],
        theme: &'a Theme,
        total_memory: u64,
    ) -> Self {
        Self {
            processes,
            theme,
            focused: true,
            total_memory,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<'a> StatefulWidget for ProcessListWidget<'a> {
    type State = ProcessListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Calculate available width for each column
        let inner_width = area.width.saturating_sub(2) as usize; // Account for borders

        // Column widths
        let name_width = inner_width.saturating_sub(22).min(20);
        let mem_width = 8;
        let bar_width = inner_width.saturating_sub(name_width + mem_width + 4);

        // Build list items with modern styling
        let items: Vec<ListItem> = self
            .processes
            .iter()
            .enumerate()
            .map(|(idx, proc)| {
                let mem_percent = if self.total_memory > 0 {
                    (proc.rss as f64 / self.total_memory as f64) * 100.0
                } else {
                    0.0
                };

                let is_selected = state.list_state.selected() == Some(idx);

                // Rank indicator for top processes
                let rank_indicator = match idx {
                    0 => Span::styled("● ", Style::default().fg(self.theme.rank_top)),
                    1..=2 => Span::styled("○ ", Style::default().fg(self.theme.rank_high)),
                    _ => Span::styled("  ", Style::default()),
                };

                // Truncate name if needed
                let name = if proc.name.len() > name_width {
                    format!("{}…", &proc.name[..name_width.saturating_sub(1)])
                } else {
                    format!("{:<width$}", proc.name, width = name_width)
                };

                // Name styling - brighter for selected, dimmer for lower ranks
                let name_style = if is_selected {
                    Style::default().fg(self.theme.selection_fg).add_modifier(Modifier::BOLD)
                } else {
                    match idx {
                        0..=2 => Style::default().fg(self.theme.fg),
                        3..=9 => Style::default().fg(self.theme.fg_dim),
                        _ => Style::default().fg(self.theme.fg_muted),
                    }
                };

                // Memory value with color based on usage
                let mem_str = format!("{:>8}", format_bytes(proc.rss));
                let mem_color = self.theme.mem_color_interpolated(mem_percent);
                let mem_style = if is_selected {
                    Style::default().fg(self.theme.selection_fg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(mem_color)
                };

                // Sleek usage bar with gradient
                let bar = create_sleek_bar(mem_percent, bar_width);
                let bar_style = if is_selected {
                    Style::default().fg(self.theme.selection_fg)
                } else {
                    Style::default().fg(mem_color)
                };

                let spans = vec![
                    rank_indicator,
                    Span::styled(name, name_style),
                    Span::raw(" "),
                    Span::styled(mem_str, mem_style),
                    Span::raw(" "),
                    Span::styled(bar, bar_style),
                ];

                ListItem::new(Line::from(spans))
            })
            .collect();

        // Update selected_pid based on current selection
        if let Some(idx) = state.list_state.selected() {
            if idx < self.processes.len() {
                state.selected_pid = Some(self.processes[idx].pid);
            }
        }

        // Modern title with sort indicator
        let title_spans = vec![
            Span::styled(" ", Style::default()),
            Span::styled("Processes", Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled(" sorted by ", self.theme.muted_style()),
            Span::styled(state.sort_mode.label(), Style::default().fg(self.theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled(" ", Style::default()),
        ];
        let title = Line::from(title_spans);

        // Build block with rounded corners feel
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(self.focused))
            .style(Style::default().bg(self.theme.bg));

        // Create list widget
        let list = List::new(items)
            .block(block)
            .highlight_style(self.theme.selected_style())
            .highlight_symbol("▸ ");

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}

/// Create a sleek usage bar with partial blocks
fn create_sleek_bar(percent: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    
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
