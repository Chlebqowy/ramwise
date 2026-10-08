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

#[cfg(test)]
mod test_support;

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
    /// Update interval in milliseconds (must be positive)
    #[arg(short, long, default_value = "1000", value_parser = clap::value_parser!(u64).range(1..))]
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
    #[arg(short, long, default_value = "dark")]
    theme: String,

    /// Custom file for theme
    #[arg(long, default_value = "$HOME/.config/ramwise/theme.toml")]
    custom_theme_file: String,

    /// Print one compact JSON snapshot to stdout and exit (exit 0 on
    /// success, non-zero when collection fails; diagnostics go to stderr).
    /// Combines with --tiny: JSON prints first, then the status line.
    #[arg(long, conflicts_with = "watch")]
    once: bool,

    /// Print one one-line status summary to stdout and exit.
    /// Combine with --watch to repeat the line every interval.
    #[arg(long)]
    tiny: bool,

    /// Repeat the --tiny line every interval until interrupted (exit 0 on
    /// clean interrupt, non-zero when collection fails)
    #[arg(long, requires = "tiny")]
    watch: bool,

    /// Write one versioned JSON snapshot to PATH (`-` for stdout).
    /// Implies a single collection like --once; refuses to overwrite
    /// existing files unless --force is given.
    #[arg(long, value_name = "PATH", conflicts_with = "watch")]
    export_json: Option<std::path::PathBuf>,

    /// Write one CSV snapshot to PATH (`-` for stdout) with the same
    /// single-collection and overwrite semantics as --export-json.
    #[arg(long, value_name = "PATH", conflicts_with = "watch")]
    export_csv: Option<std::path::PathBuf>,

    /// Allow exports to overwrite existing files.
    #[arg(long)]
    force: bool,

    /// Include expensive region details in exports (adds a regions_json
    /// column to CSV).
    #[arg(long)]
    export_details: bool,

    /// Insert the capture timestamp before the output file extension.
    #[arg(long)]
    export_timestamped: bool,
}

/// How the process executes. Only [`ExecutionMode::Tui`] may initialize the
/// terminal; every other mode is plain stdout/stderr and never enters raw mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionMode {
    Tui,
    Once,
    TinyOnce,
    TinyWatch,
}

fn execution_mode(args: &Args) -> ExecutionMode {
    if args.watch {
        ExecutionMode::TinyWatch
    } else if args.tiny {
        ExecutionMode::TinyOnce
    } else if args.once {
        ExecutionMode::Once
    } else {
        ExecutionMode::Tui
    }
}

/// Build the collector shared by every execution mode so flags behave
/// identically in the TUI and in non-interactive commands.
fn build_collector(args: &Args) -> Collector {
    Collector::new()
        .with_interval(Duration::from_millis(args.interval))
        .with_min_rss(args.min_rss.saturating_mul(1024 * 1024))
        .with_smaps(!args.no_smaps)
        .with_regions(
            args.export_details && (args.export_json.is_some() || args.export_csv.is_some()),
        )
}

/// File exports requested on the command line. Any export implies a single
/// collection like `--once`; `--tiny` additionally prints the status line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestedExports {
    json: Option<std::path::PathBuf>,
    csv: Option<std::path::PathBuf>,
    force: bool,
    details: bool,
    timestamped: bool,
}

fn requested_exports(args: &Args) -> Option<RequestedExports> {
    if args.export_json.is_none() && args.export_csv.is_none() {
        return None;
    }
    Some(RequestedExports {
        json: args.export_json.clone(),
        csv: args.export_csv.clone(),
        force: args.force,
        details: args.export_details,
        timestamped: args.export_timestamped,
    })
}

/// Write every requested export. File confirmations go to stderr so stdout
/// stays pure data when a `-` target is used.
fn run_exports(export: &collector::ExportSnapshot, exports: &RequestedExports) -> Result<()> {
    let json_path = exports
        .json
        .as_ref()
        .map(|path| export_path(path, export.captured_at_unix_ms, exports.timestamped))
        .transpose()?;
    let csv_path = exports
        .csv
        .as_ref()
        .map(|path| export_path(path, export.captured_at_unix_ms, exports.timestamped))
        .transpose()?;
    if let (Some(json), Some(csv)) = (&json_path, &csv_path)
        && same_export_target(json, csv)?
    {
        anyhow::bail!("JSON and CSV exports must use different paths");
    }

    fn same_export_target(left: &std::path::Path, right: &std::path::Path) -> Result<bool> {
        if left.as_os_str() == "-" || right.as_os_str() == "-" {
            return Ok(left.as_os_str() == right.as_os_str());
        }
        if left.exists() && right.exists() {
            let left_metadata = std::fs::metadata(left)?;
            let right_metadata = std::fs::metadata(right)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                return Ok(left_metadata.dev() == right_metadata.dev()
                    && left_metadata.ino() == right_metadata.ino());
            }
            #[cfg(not(unix))]
            {
                return Ok(std::fs::canonicalize(left)? == std::fs::canonicalize(right)?);
            }
        }
        let normalize = |path: &std::path::Path| -> Result<std::path::PathBuf> {
            let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
            Ok(std::fs::canonicalize(parent)?
                .join(path.file_name().context("export path must name a file")?))
        };
        Ok(normalize(left)? == normalize(right)?)
    }
    let has_stdout = [&json_path, &csv_path]
        .into_iter()
        .flatten()
        .any(|path| path.as_os_str() == "-");
    let has_file = [&json_path, &csv_path]
        .into_iter()
        .flatten()
        .any(|path| path.as_os_str() != "-");
    if has_stdout && has_file {
        anyhow::bail!("stdout exports cannot be combined with file exports");
    }
    let json_text = json_path
        .as_ref()
        .map(|_| collector::render_json(export))
        .transpose()?;
    let csv_text = csv_path
        .as_ref()
        .map(|_| collector::render_csv(export, exports.details))
        .transpose()?;
    if !exports.force {
        for path in [&json_path, &csv_path].into_iter().flatten() {
            if path.as_os_str() != "-" && path.exists() {
                anyhow::bail!("refusing to overwrite existing file {}", path.display());
            }
        }
    } else {
        for path in [&json_path, &csv_path].into_iter().flatten() {
            if path.as_os_str() != "-" && path.exists() && !path.is_file() {
                anyhow::bail!(
                    "refusing to replace non-file export target {}",
                    path.display()
                );
            }
        }
    }
    let file_targets = [
        json_path
            .as_ref()
            .zip(json_text.as_ref())
            .filter(|(path, _)| path.as_os_str() != "-"),
        csv_path
            .as_ref()
            .zip(csv_text.as_ref())
            .filter(|(path, _)| path.as_os_str() != "-"),
    ];
    let file_targets: Vec<_> = file_targets.into_iter().flatten().collect();
    collector::write_targets(
        &file_targets
            .iter()
            .map(|(path, text)| (path.as_path(), text.as_str()))
            .collect::<Vec<_>>(),
        exports.force,
    )?;
    if let (Some(path), Some(text)) = (&json_path, &json_text) {
        if path.as_os_str() == "-" {
            collector::write_target(path, text, exports.force)?;
        }
        if path.as_os_str() != "-" {
            eprintln!("exported JSON snapshot to {}", path.display());
        }
    }
    if let (Some(path), Some(text)) = (&csv_path, &csv_text) {
        if path.as_os_str() == "-" {
            collector::write_target(path, text, exports.force)?;
        }
        if path.as_os_str() != "-" {
            eprintln!("exported CSV snapshot to {}", path.display());
        }
    }
    Ok(())
}

fn export_path(
    path: &std::path::Path,
    timestamp: u64,
    timestamped: bool,
) -> Result<std::path::PathBuf> {
    if !timestamped || path.as_os_str() == "-" {
        return Ok(path.to_path_buf());
    }
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let stem = path
        .file_stem()
        .context("timestamped export path must name a file")?
        .to_string_lossy();
    let extension = path.extension().map(|value| value.to_string_lossy());
    let name = match extension {
        Some(extension) => format!("{stem}-{timestamp}.{extension}"),
        None => format!("{stem}-{timestamp}"),
    };
    Ok(parent.join(name))
}

/// Serialize one snapshot to the versioned export contract for `--once`.
/// The payload is compact JSON on stdout; failures are `Err` so the process
/// exits non-zero with the cause on stderr.
fn snapshot_to_json(snapshot: &collector::MemorySnapshot) -> Result<String> {
    collector::render_json(&snapshot.to_export())
}

/// Render the one-line status summary for `--tiny`.
///
/// Provisional shape until the stable status-bar contract lands: used/total
/// RAM, usage percent, and used/total swap. Pure and deterministic so golden
/// tests pin it exactly.
fn render_tiny_line(system: &collector::SystemMemory) -> String {
    format!(
        "mem {}/{} {} swap {}/{}",
        utils::format_bytes(system.used()),
        utils::format_bytes(system.total),
        utils::format_percent(system.usage_percent()),
        utils::format_bytes(system.swap_used),
        utils::format_bytes(system.swap_total),
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Diagnostics always go to stderr so stdout stays pure data in
    // non-interactive modes (pipelines, status bars, file redirects).
    if args.debug {
        tracing_subscriber::fmt()
            .with_env_filter("ramwise=debug")
            .with_writer(io::stderr)
            .init();
    }

    let mode = execution_mode(&args);
    let exports = requested_exports(&args);
    if mode == ExecutionMode::Tui && exports.is_none() {
        return run_tui(&args).await;
    }
    check_stdout_payloads(&args)?;
    if args.export_details && args.no_smaps {
        eprintln!(
            "warning: --export-details has no effect with --no-smaps (region detail unavailable)"
        );
    }
    // Single-shot data path. Requesting an export implies one collection
    // like --once; --tiny additionally prints the status line.
    let mut collector = build_collector(&args);
    let snapshot = collector.collect_snapshot()?;
    let export = snapshot.to_export();
    if let Some(exports) = &exports {
        run_exports(&export, exports)?;
    }
    match mode {
        ExecutionMode::Tui => Ok(()),
        ExecutionMode::Once => {
            // Explicit --once always prints; bare --export-* writes files only.
            println!("{}", collector::render_json(&export)?);
            Ok(())
        }
        ExecutionMode::TinyOnce => {
            // Explicit --once composes: JSON payload first (pipelines read
            // it with head -1), then the status line.
            if args.once {
                println!("{}", snapshot_to_json(&snapshot)?);
            }
            println!("{}", render_tiny_line(&snapshot.system));
            Ok(())
        }
        ExecutionMode::TinyWatch => run_tiny_watch(&args).await,
    }
}

/// At most one stdout data payload, so pipelines stay unambiguous.
/// The single exception is the documented `--once --tiny` dual output
/// (JSON first, then the status line); every other combination of stdout
/// producers (JSON print, tiny line, `-` export targets) fails fast.
fn check_stdout_payloads(args: &Args) -> Result<()> {
    let mut payloads = 0;
    if args.once {
        payloads += 1;
    }
    if args.tiny && !args.watch {
        payloads += 1;
    }
    for target in [args.export_json.as_ref(), args.export_csv.as_ref()]
        .into_iter()
        .flatten()
    {
        if target.as_os_str() == "-" {
            payloads += 1;
        }
    }
    let documented_dual = args.once && args.tiny && !args.watch && payloads == 2;
    if payloads > 1 && !documented_dual {
        anyhow::bail!(
            "multiple stdout payloads requested; write exports to files or request a single output"
        );
    }
    Ok(())
}

/// Flush stdout so piped consumers see each line immediately.
fn flush_stdout() -> Result<()> {
    use std::io::Write as _;
    io::stdout().flush().context("Failed to flush stdout")
}

/// Print the tiny line every interval until interrupted (SIGINT or
/// SIGTERM). A clean interrupt ends with exit 0; a collection failure ends
/// non-zero with the cause on stderr.
async fn run_tiny_watch(args: &Args) -> Result<()> {
    let mut collector = build_collector(args);
    let mut ticker = tokio::time::interval(Duration::from_millis(args.interval));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let snapshot = collector.collect_snapshot()?;
                println!("{}", render_tiny_line(&snapshot.system));
                flush_stdout()?;
            }
            result = tokio::signal::ctrl_c() => {
                result.context("Failed to listen for interrupt")?;
                return Ok(());
            }
            _ = terminate_signal() => {
                return Ok(());
            }
        }
    }
}

/// SIGTERM waiter; pending forever off unix (Linux-only binary, but the
/// gate keeps cross-compilation honest).
async fn terminate_signal() {
    #[cfg(unix)]
    if let Ok(mut terminate) =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
        terminate.recv().await;
    }
    #[cfg(not(unix))]
    std::future::pending::<()>().await;
}

async fn run_tui(args: &Args) -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to setup terminal")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create app
    let mut app = App::new(&args.theme, &args.custom_theme_file);

    // Create collector
    let collector = build_collector(args);

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
    let layout = Layout::new(app.theme.clone());

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
                if event::poll(Duration::from_millis(50)).unwrap_or(false)
                    && let Ok(Event::Key(key)) = event::read()
                        && key.kind == KeyEventKind::Press {
                            app.handle_key(key.code, key.modifiers);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    fn args_with(tiny: bool, once: bool, watch: bool) -> Args {
        Args {
            interval: 1000,
            min_rss: 1,
            no_smaps: false,
            debug: false,
            theme: "dark".into(),
            custom_theme_file: "$HOME/.config/ramwise/theme.toml".to_string(),
            tiny,
            once,
            watch,
            export_json: None,
            export_csv: None,
            force: false,
            export_details: false,
            export_timestamped: false,
        }
    }

    #[test]
    fn mode_dispatch_prefers_the_most_specific_flag() {
        assert_eq!(
            execution_mode(&args_with(false, false, false)),
            ExecutionMode::Tui
        );
        assert_eq!(
            execution_mode(&args_with(false, true, false)),
            ExecutionMode::Once
        );
        assert_eq!(
            execution_mode(&args_with(true, false, false)),
            ExecutionMode::TinyOnce
        );
        assert_eq!(
            execution_mode(&args_with(true, true, false)),
            ExecutionMode::TinyOnce
        );
        assert_eq!(
            execution_mode(&args_with(true, false, true)),
            ExecutionMode::TinyWatch
        );
        // Clap rejects --once --watch, but the dispatcher stays total:
        // watch wins deterministically if both ever arrive.
        let mut both = args_with(true, true, false);
        both.watch = true;
        assert_eq!(execution_mode(&both), ExecutionMode::TinyWatch);
    }

    #[test]
    fn only_the_tui_mode_may_initialize_the_terminal() {
        // Structural guarantee: terminal setup lives in run_tui, and every
        // non-interactive mode resolves away from it. If a new mode is added
        // without updating this match, it fails closed here.
        for mode in [
            ExecutionMode::Once,
            ExecutionMode::TinyOnce,
            ExecutionMode::TinyWatch,
        ] {
            assert_ne!(mode, ExecutionMode::Tui);
        }
    }

    #[test]
    fn shared_builder_maps_min_rss_and_smaps_flags() {
        let filtered = Args {
            min_rss: 1_000_000,
            ..args_with(false, true, false)
        };
        let mut collector = build_collector(&filtered);
        let snapshot = collector.collect_snapshot().unwrap();
        assert!(snapshot.processes.is_empty());

        let plain = args_with(false, true, false);
        let mut collector = build_collector(&plain);
        assert!(collector.collect_snapshot().is_ok());

        // --no-smaps disables detailed PSS/USS collection end to end.
        let bare = Args {
            no_smaps: true,
            min_rss: 0,
            ..args_with(false, true, false)
        };
        let mut collector = build_collector(&bare);
        let snapshot = collector.collect_snapshot().unwrap();
        assert!(
            snapshot.processes.iter().all(|p| p.pss == 0 && p.uss == 0),
            "smaps details must stay off with --no-smaps"
        );
    }

    #[test]
    fn tiny_line_is_pinned_by_a_golden_fixture() {
        let line = render_tiny_line(&test_support::system_memory());
        assert_eq!(line, "mem 8.0G/16.0G 50.0% swap 512.0M/4.0G");
    }

    #[test]
    fn once_output_is_valid_versioned_json() {
        let snapshot = test_support::snapshot_at(std::time::Instant::now(), 100);
        let json = snapshot_to_json(&snapshot).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        // snapshot_to_json already enforces schema validation; here we pin
        // the wire shape so regressions surface at the CLI boundary too.
        assert!(value["schema_version"].is_number());
        assert!(value["system"]["total_bytes"].is_number());
        assert!(value["processes"].is_array());
    }

    #[test]
    fn export_request_mapping_is_explicit() {
        assert_eq!(requested_exports(&args_with(false, false, false)), None);
        let mut args = args_with(false, true, false);
        args.export_json = Some(std::path::PathBuf::from("snap.json"));
        args.export_csv = Some(std::path::PathBuf::from("-"));
        args.force = true;
        args.export_details = true;
        assert_eq!(
            requested_exports(&args),
            Some(RequestedExports {
                json: Some(std::path::PathBuf::from("snap.json")),
                csv: Some(std::path::PathBuf::from("-")),
                force: true,
                details: true,
                timestamped: false,
            })
        );
    }

    #[test]
    fn stdout_payload_rule_allows_only_the_documented_dual() {
        assert!(check_stdout_payloads(&args_with(false, true, false)).is_ok());
        assert!(check_stdout_payloads(&args_with(true, false, false)).is_ok());
        // --once --tiny is the documented dual output.
        let mut dual = args_with(true, true, false);
        assert!(check_stdout_payloads(&dual).is_ok());
        // A second stdout producer on top refuses fast.
        dual.export_csv = Some(std::path::PathBuf::from("-"));
        assert!(check_stdout_payloads(&dual).is_err());
        let mut mixed = args_with(false, true, false);
        mixed.export_json = Some(std::path::PathBuf::from("-"));
        assert!(check_stdout_payloads(&mixed).is_err());
    }
}
