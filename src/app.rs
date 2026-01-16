//! Application state and main logic

use crossterm::event::{KeyCode, KeyModifiers};

use crate::analyzer::Analyzer;
use crate::collector::{MemorySnapshot, ProcessMemory};
use crate::history::HistoryBuffer;
use crate::ui::widgets::{ProcessListState, SortMode};
use crate::ui::Theme;

/// Focus state for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    ProcessList,
    DetailPanel,
    GraphPanel,
    InsightsPanel,
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::ProcessList => Focus::DetailPanel,
            Focus::DetailPanel => Focus::GraphPanel,
            Focus::GraphPanel => Focus::InsightsPanel,
            Focus::InsightsPanel => Focus::ProcessList,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Focus::ProcessList => Focus::InsightsPanel,
            Focus::DetailPanel => Focus::ProcessList,
            Focus::GraphPanel => Focus::DetailPanel,
            Focus::InsightsPanel => Focus::GraphPanel,
        }
    }
}

/// Main application state
pub struct App {
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current focus
    pub focus: Focus,
    /// Theme
    pub theme: Theme,
    /// Latest memory snapshot
    pub snapshot: Option<MemorySnapshot>,
    /// History buffer for trends
    pub history: HistoryBuffer,
    /// Analyzer for insights
    pub analyzer: Analyzer,
    /// Process list state
    pub process_list_state: ProcessListState,
    /// Whether to show help overlay
    pub show_help: bool,
    /// Sorted processes (cached)
    sorted_processes: Vec<ProcessMemory>,
}

impl App {
    /// Create a new application
    pub fn new() -> Self {
        Self {
            should_quit: false,
            focus: Focus::ProcessList,
            theme: Theme::dark(),
            snapshot: None,
            history: HistoryBuffer::default_5min(),
            analyzer: Analyzer::new(),
            process_list_state: ProcessListState::new(),
            show_help: false,
            sorted_processes: Vec::new(),
        }
    }

    /// Update with a new memory snapshot
    pub fn update(&mut self, snapshot: MemorySnapshot) {
        // Add to history
        self.history.push(&snapshot);

        // Run analyzer
        self.analyzer.analyze(&snapshot, &self.history);

        // Update sorted processes
        self.update_sorted_processes(&snapshot);

        // Update selected process
        self.update_selection();

        // Store snapshot
        self.snapshot = Some(snapshot);
    }

    /// Update sorted process list based on current sort mode
    fn update_sorted_processes(&mut self, snapshot: &MemorySnapshot) {
        self.sorted_processes = snapshot.processes.clone();

        match self.process_list_state.sort_mode {
            SortMode::Rss => {
                self.sorted_processes.sort_by(|a, b| b.rss.cmp(&a.rss));
            }
            SortMode::Pss => {
                self.sorted_processes.sort_by(|a, b| b.pss.cmp(&a.pss));
            }
            SortMode::Private => {
                self.sorted_processes.sort_by(|a, b| b.private.cmp(&a.private));
            }
            SortMode::Name => {
                self.sorted_processes.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortMode::Pid => {
                self.sorted_processes.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
        }
    }

    /// Update selection after sort change
    fn update_selection(&mut self) {
        // Try to keep the same process selected
        if let Some(selected_pid) = self.process_list_state.selected_pid {
            if let Some(idx) = self.sorted_processes.iter().position(|p| p.pid == selected_pid) {
                self.process_list_state.list_state.select(Some(idx));
                return;
            }
        }

        // Otherwise, ensure selection is valid
        if let Some(selected) = self.process_list_state.list_state.selected() {
            if selected >= self.sorted_processes.len() && !self.sorted_processes.is_empty() {
                self.process_list_state.list_state.select(Some(0));
            }
        }
    }

    /// Re-sort existing processes (for sort mode change)
    fn resort_processes(&mut self) {
        match self.process_list_state.sort_mode {
            SortMode::Rss => {
                self.sorted_processes.sort_by(|a, b| b.rss.cmp(&a.rss));
            }
            SortMode::Pss => {
                self.sorted_processes.sort_by(|a, b| b.pss.cmp(&a.pss));
            }
            SortMode::Private => {
                self.sorted_processes.sort_by(|a, b| b.private.cmp(&a.private));
            }
            SortMode::Name => {
                self.sorted_processes.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortMode::Pid => {
                self.sorted_processes.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
        }
        self.update_selection();
    }

    /// Get sorted processes
    pub fn processes(&self) -> &[ProcessMemory] {
        &self.sorted_processes
    }

    /// Get currently selected process
    pub fn selected_process(&self) -> Option<&ProcessMemory> {
        let idx = self.process_list_state.list_state.selected()?;
        self.sorted_processes.get(idx)
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Global keys
        match (key, modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            (KeyCode::Char('?'), _) => {
                self.show_help = !self.show_help;
                return;
            }
            (KeyCode::Esc, _) => {
                if self.show_help {
                    self.show_help = false;
                    return;
                }
            }
            (KeyCode::Tab, KeyModifiers::SHIFT) => {
                self.focus = self.focus.prev();
                return;
            }
            (KeyCode::Tab, _) => {
                self.focus = self.focus.next();
                return;
            }
            _ => {}
        }

        // Panel-specific keys
        match self.focus {
            Focus::ProcessList => self.handle_process_list_key(key),
            Focus::DetailPanel => {}
            Focus::GraphPanel => {}
            Focus::InsightsPanel => {}
        }
    }

    fn handle_process_list_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.process_list_state.select_previous(self.sorted_processes.len());
                self.update_selected_pid();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.process_list_state.select_next(self.sorted_processes.len());
                self.update_selected_pid();
            }
            KeyCode::Char('s') => {
                self.process_list_state.cycle_sort();
                self.resort_processes();
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if !self.sorted_processes.is_empty() {
                    self.process_list_state.list_state.select(Some(0));
                    self.update_selected_pid();
                }
            }
            KeyCode::End | KeyCode::Char('G') => {
                if !self.sorted_processes.is_empty() {
                    self.process_list_state.list_state.select(Some(self.sorted_processes.len() - 1));
                    self.update_selected_pid();
                }
            }
            _ => {}
        }
    }

    fn update_selected_pid(&mut self) {
        if let Some(idx) = self.process_list_state.list_state.selected() {
            if let Some(proc) = self.sorted_processes.get(idx) {
                self.process_list_state.selected_pid = Some(proc.pid);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
