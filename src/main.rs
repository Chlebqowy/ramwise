//! ramwise - Intelligent RAM usage visualizer for Arch Linux
//!
//! A TUI application that provides deep memory introspection,
//! intelligent insights, and beautiful visualization.

mod analyzer;
mod app;
mod collector;
mod history;
mod process_control;
mod ui;
mod utils;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tokio::sync::mpsc;

use app::{ActionStatus, ActionStatusKind, App, Focus};
use collector::Collector;
use ui::Layout;
use ui::widgets::{
    DetailPanelWidget, GraphWidget, HeaderWidget, InsightsPanelWidget, ProcessListWidget,
};

/// Intelligent RAM usage visualizer for Arch Linux
#[derive(Parser, Debug)]
#[command(name = "ramwise")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Update interval in milliseconds
    #[arg(short, long, default_value = "1000")]
    interval: u64,

    /// Minimum process RSS to display (in MB)
    #[arg(short, long, default_value = "1")]
    min_rss: u64,

    /// Disable smaps collection (faster but less detailed)
    #[arg(long)]
    no_smaps: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Theme (by default light or dark)
    #[arg(long, default_value = "dark")]
    theme: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging if debug mode
    if args.debug {
        tracing_subscriber::fmt()
            .with_env_filter("ramwise=debug")
            .init();
    }

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to setup terminal")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create app
    let mut app = App::new(&args.theme);

    // Create collector
    let collector = Collector::new()
        .with_interval(Duration::from_millis(args.interval))
        .with_min_rss(args.min_rss * 1024 * 1024)
        .with_smaps(!args.no_smaps);

    // Create channel for snapshots
    let (tx, mut rx) = mpsc::channel(2);

    // Spawn collector task
    let collector_handle = tokio::spawn(async move {
        if let Err(e) = collector.run(tx).await {
            tracing::error!("Collector error: {}", e);
        }
    });

    // Main event loop
    let result = run_app(&mut terminal, &mut app, &mut rx).await;

    // Cleanup
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to cleanup terminal")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    // Abort collector
    collector_handle.abort();

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::Receiver<collector::MemorySnapshot>,
) -> Result<()> {
    let layout = Layout::new();

    loop {
        app.prune_transient_state();

        // Draw
        terminal.draw(|frame| {
            let areas = layout.calculate(frame.area());

            // Header
            if let Some(snapshot) = &app.snapshot {
                let header = HeaderWidget::new(&snapshot.system, &app.theme);
                frame.render_widget(header, areas.header);
            } else {
                let loading =
                    Paragraph::new(" ramwise - Loading...").style(app.theme.header_style());
                frame.render_widget(loading, areas.header);
            }

            // Process list
            if let Some(snapshot) = &app.snapshot {
                let total_mem = snapshot.system.total;
                let focus = app.focus;
                let processes = app.processes().to_vec();
                let theme = app.theme.clone();

                let process_list = ProcessListWidget::new(&processes, &theme, total_mem)
                    .focused(focus == Focus::ProcessList);

                frame.render_stateful_widget(
                    process_list,
                    areas.left_panel,
                    &mut app.process_list_state,
                );
            } else {
                let block = Block::default()
                    .title(" PROCESSES ")
                    .borders(Borders::ALL)
                    .border_style(app.theme.border_style(app.focus == Focus::ProcessList));
                frame.render_widget(block, areas.left_panel);
            }

            // Detail panel
            let detail = DetailPanelWidget::new(app.selected_process(), &app.theme)
                .focused(app.focus == Focus::DetailPanel);
            frame.render_widget(detail, areas.detail_panel);

            // Graph panel
            let graph = GraphWidget::new(&app.history, &app.theme)
                .selected_pid(app.process_list_state.selected_pid)
                .focused(app.focus == Focus::GraphPanel);
            frame.render_widget(graph, areas.graph_panel);

            // Insights panel
            let insights = InsightsPanelWidget::new(app.analyzer.insights(), &app.theme)
                .focused(app.focus == Focus::InsightsPanel);
            frame.render_widget(insights, areas.bottom);

            // Help overlay
            if app.show_help {
                render_help_overlay(frame, &app.theme);
            }

            // Kill confirmation overlay
            if app.show_kill_confirm {
                render_kill_confirm_overlay(frame, app);
            }

            if let Some(status) = app.action_status.as_ref() {
                render_action_status(frame, &app.theme, status);
            }
        })?;

        // Handle events (with timeout for data updates)
        tokio::select! {
            // New snapshot from collector
            Some(snapshot) = rx.recv() => {
                app.update(snapshot);
            }

            // Keyboard/mouse events
            _ = async {
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            app.handle_key(key.code, key.modifiers);
                        }
                    }
                }
            } => {}
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn render_help_overlay(frame: &mut ratatui::Frame, theme: &ui::Theme) {
    let area = frame.area();

    // Center a help box
    let help_width = 50.min(area.width - 4);
    let help_height = 18.min(area.height - 4);
    let x = (area.width - help_width) / 2;
    let y = (area.height - help_height) / 2;

    let help_area = ratatui::layout::Rect::new(x, y, help_width, help_height);

    // Clear background
    frame.render_widget(Clear, help_area);

    let help_text = r#"
  KEYBOARD SHORTCUTS

  Navigation:
    j/k or ↑/↓   Move selection
    Tab          Cycle focus
    Shift+Tab    Reverse cycle

  Process List:
    s            Cycle sort mode
    g            Go to top
    G            Go to bottom
    x            Send SIGTERM
    X            Confirm + send SIGKILL

  General:
    ?            Toggle this help
    q            Quit
    Ctrl+C       Force quit

  Press ESC or ? to close
"#;

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(theme.border_style(true))
                .style(theme.base_style()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help, help_area);
}

fn render_kill_confirm_overlay(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();
    let width = 62.min(area.width.saturating_sub(4));
    let height = 7.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, modal_area);

    let label = if let Some(proc_) = app.selected_process() {
        format!("Kill {} (PID {}) with SIGKILL?", proc_.name, proc_.pid)
    } else {
        "No process selected".to_string()
    };

    let message = vec![
        Line::from(Span::styled(label, Style::default().fg(app.theme.warning))),
        Line::from(""),
        Line::from("Press Enter to confirm, Esc to cancel"),
    ];

    let paragraph = Paragraph::new(message).alignment(Alignment::Center).block(
        Block::default()
            .title(" Confirm Kill ")
            .borders(Borders::ALL)
            .border_style(app.theme.border_style(true))
            .style(app.theme.base_style()),
    );

    frame.render_widget(paragraph, modal_area);
}

fn render_action_status(frame: &mut ratatui::Frame, theme: &ui::Theme, status: &ActionStatus) {
    let area = frame.area();
    let width = 70.min(area.width.saturating_sub(4));
    let height = 3;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = area.height.saturating_sub(height + 1);
    let toast_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, toast_area);

    let color = match status.kind {
        ActionStatusKind::Success => theme.success,
        ActionStatusKind::Warning => theme.warning,
        ActionStatusKind::Error => theme.error,
    };
    let label = match status.kind {
        ActionStatusKind::Success => "OK",
        ActionStatusKind::Warning => "WARN",
        ActionStatusKind::Error => "ERR",
    };

    let line = Line::from(vec![
        Span::styled(format!("[{}] ", label), Style::default().fg(color)),
        Span::styled(&status.message, Style::default().fg(theme.fg)),
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Left).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .style(theme.base_style()),
    );

    frame.render_widget(paragraph, toast_area);
}
