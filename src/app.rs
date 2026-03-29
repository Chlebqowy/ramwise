//! Application state and main logic

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};

use crate::analyzer::Analyzer;
use crate::collector::{MemorySnapshot, ProcessMemory};
use crate::history::HistoryBuffer;
use crate::process_control::{SignalAction, SignalResult, send_signal};
use crate::ui::Theme;
use crate::ui::widgets::{ProcessListState, SortMode};

/// Focus state for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    ProcessList,
    DetailPanel,
    InsightsPanel,
    GraphPanel,
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::ProcessList => Focus::DetailPanel,
            Focus::DetailPanel => Focus::InsightsPanel,
            Focus::InsightsPanel => Focus::GraphPanel,
            Focus::GraphPanel => Focus::ProcessList,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Focus::ProcessList => Focus::GraphPanel,
            Focus::DetailPanel => Focus::ProcessList,
            Focus::GraphPanel => Focus::InsightsPanel,
            Focus::InsightsPanel => Focus::DetailPanel,
        }
    }
}

/// Status severity for action feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionStatusKind {
    Success,
    Warning,
    Error,
}

/// Transient status message shown in UI after process actions.
#[derive(Debug, Clone)]
pub struct ActionStatus {
    pub kind: ActionStatusKind,
    pub message: String,
    pub expires_at: Instant,
}

const STATUS_TTL: Duration = Duration::from_secs(4);

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
    /// Whether SIGKILL confirmation dialog is visible
    pub show_kill_confirm: bool,
    /// Transient status for process action feedback
    pub action_status: Option<ActionStatus>,
    /// Sorted processes (cached)
    sorted_processes: Vec<ProcessMemory>,
}

impl App {
    /// Create a new application
    pub fn new(theme_name: &str) -> Self {
        let mut theme = Theme::dark();
        if theme_name == "light" {
            theme = Theme::light();
        } else if theme_name == "dark" {
            theme = Theme::dark();
        } else if theme_name == "green" {
            theme = Theme::green();
        } else {
            eprintln!("Invalid theme: {theme_name}. Did you add it to app.rs? Using dark as fallback.");
        }
        Self {
            should_quit: false,
            focus: Focus::ProcessList,
            theme,
            snapshot: None,
            history: HistoryBuffer::default_5min(),
            analyzer: Analyzer::new(),
            process_list_state: ProcessListState::new(),
            show_help: false,
            show_kill_confirm: false,
            action_status: None,
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
                self.sorted_processes
                    .sort_by(|a, b| b.private.cmp(&a.private));
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
            if let Some(idx) = self
                .sorted_processes
                .iter()
                .position(|p| p.pid == selected_pid)
            {
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
                self.sorted_processes
                    .sort_by(|a, b| b.private.cmp(&a.private));
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
        if self.show_kill_confirm {
            self.handle_kill_confirm_key(key);
            return;
        }

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
                self.process_list_state
                    .select_previous(self.sorted_processes.len());
                self.update_selected_pid();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.process_list_state
                    .select_next(self.sorted_processes.len());
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
                    self.process_list_state
                        .list_state
                        .select(Some(self.sorted_processes.len() - 1));
                    self.update_selected_pid();
                }
            }
            KeyCode::Char('x') => {
                self.signal_selected_process(SignalAction::Terminate);
            }
            KeyCode::Char('X') => {
                if self.selected_process().is_none() {
                    self.set_status(ActionStatusKind::Warning, "No process selected");
                } else {
                    self.show_kill_confirm = true;
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

    /// Remove expired transient messages.
    pub fn prune_transient_state(&mut self) {
        if let Some(status) = &self.action_status {
            if Instant::now() >= status.expires_at {
                self.action_status = None;
            }
        }
    }

    fn handle_kill_confirm_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                self.show_kill_confirm = false;
                self.signal_selected_process(SignalAction::Kill);
            }
            KeyCode::Esc => {
                self.show_kill_confirm = false;
                self.set_status(ActionStatusKind::Warning, "Kill action canceled");
            }
            _ => {}
        }
    }

    fn signal_selected_process(&mut self, action: SignalAction) {
        let Some(process) = self.selected_process() else {
            self.set_status(ActionStatusKind::Warning, "No process selected");
            return;
        };

        let pid = process.pid;
        let name = process.name.clone();
        let result = send_signal(pid, action);
        self.apply_signal_result(result, action, pid, &name);
    }

    fn apply_signal_result(
        &mut self,
        result: SignalResult,
        action: SignalAction,
        pid: i32,
        name: &str,
    ) {
        match result {
            SignalResult::Sent => {
                self.set_status(
                    ActionStatusKind::Success,
                    format!("{} sent to {} (PID {})", action.as_label(), name, pid),
                );
            }
            SignalResult::NotFound => {
                self.set_status(
                    ActionStatusKind::Warning,
                    format!("Process {} (PID {}) no longer exists", name, pid),
                );
            }
            SignalResult::PermissionDenied => {
                self.set_status(
                    ActionStatusKind::Error,
                    format!("Permission denied for {} (PID {})", name, pid),
                );
            }
            SignalResult::InvalidTarget => {
                self.set_status(
                    ActionStatusKind::Warning,
                    format!("Refusing to signal protected PID {}", pid),
                );
            }
            SignalResult::Failed(err) => {
                self.set_status(
                    ActionStatusKind::Error,
                    format!("Failed to send {}: {}", action.as_label(), err),
                );
            }
        }
    }

    fn set_status(&mut self, kind: ActionStatusKind, message: impl Into<String>) {
        self.action_status = Some(ActionStatus {
            kind,
            message: message.into(),
            expires_at: Instant::now() + STATUS_TTL,
        });
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new("dark")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_confirm_toggles_on_selection() {
        let mut app = App::new();
        app.sorted_processes.push(ProcessMemory {
            pid: 4242,
            name: "worker".to_string(),
            ..ProcessMemory::default()
        });
        app.process_list_state.list_state.select(Some(0));

        app.handle_key(KeyCode::Char('X'), KeyModifiers::NONE);
        assert!(app.show_kill_confirm);
    }

    #[test]
    fn kill_confirm_esc_cancels_modal() {
        let mut app = App::new();
        app.show_kill_confirm = true;

        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);

        assert!(!app.show_kill_confirm);
        assert!(app.action_status.is_some());
    }

    #[test]
    fn kill_key_without_selection_sets_warning() {
        let mut app = App::new();
        app.handle_key(KeyCode::Char('x'), KeyModifiers::NONE);

        assert!(matches!(
            app.action_status.as_ref().map(|s| s.kind),
            Some(ActionStatusKind::Warning)
        ));
    }
}
