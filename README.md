# ramwise
> Your memory's wise advisor - Intelligent RAM usage visualizer for Arch Linux

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

**ramwise** is a terminal-based RAM usage visualizer that goes beyond basic memory monitoring. It provides deep memory introspection, intelligent leak detection, and beautiful visualization all in a lightweight TUI application.

## Features

- **Deep Memory Introspection** - See RSS, PSS, USS, shared/private breakdown per process
- **Intelligent Insights** - Rule-based analysis detects memory leaks, hogs, and anomalies
- **Beautiful TUI** - Modern interface with graphs, colors, and intuitive navigation
- **Minimal Footprint** - Written in Rust for maximum efficiency
- **Real-time Updates** - Live monitoring with configurable refresh rate
- **Process Control** - Stop (`SIGTERM`) or kill (`SIGKILL`) selected processes directly from the TUI

### Snapshot exports

The collector exposes a versioned `ExportSnapshot` contract for integrations and
future export formats. It uses wall-clock Unix milliseconds, explicit byte/count
units, collector metadata, and capability markers. Runtime-only monotonic
`Instant` values are intentionally not serialized; unavailable features remain
explicit in the exported capability map.

## Screenshots

![Screenshot](public/screenshot.png)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/Duckaet/ramwise
cd ramwise

# Build release binary
cargo build --release

# Install (optional)
sudo cp target/release/ramwise /usr/local/bin/
```

## Usage

```bash
# Run with default settings (1s refresh, 1MB minimum RSS)
ramwise

# Custom refresh interval (500ms)
ramwise --interval 500

# Show all processes (including small ones)
ramwise --min-rss 0

# Disable smaps collection (faster but less detailed)
ramwise --no-smaps

# Enable debug logging
ramwise --debug

# Use light mode
ramwise -t light

# Use config at /home/user/.config/ramwise/theme.toml
ramwise --custom_theme_file $HOME/.config/ramwise/theme.toml 
```

### Capture snapshots

Capture one JSON snapshot to stdout, or write JSON/CSV files without starting
the TUI:

```bash
# Machine-readable JSON on stdout
ramwise --once > snapshot.json

# Refuse to overwrite existing files
ramwise --export-json snapshot.json --export-csv snapshot.csv

# Replace existing files explicitly
ramwise --export-json snapshot.json --force

# Include per-mapping details in the CSV and timestamp the filename
ramwise --export-csv capture.csv --export-details --export-timestamped

# Stream CSV to stdout
ramwise --export-csv -
```

Export failures and diagnostics are written to stderr; JSON and CSV payloads
remain on stdout only when `-` is used.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j/k` or `↑/↓` | Navigate process list |
| `Tab` | Cycle focus between panels |
| `Shift+Tab` | Reverse cycle focus |
| `s` | Cycle sort mode (RSS/PSS/Private/Name/PID) |
| `g` | Go to top of list |
| `G` | Go to bottom of list |
| `x` | Send `SIGTERM` to selected process |
| `X` | Confirm and send `SIGKILL` to selected process |
| `?` | Toggle help overlay |
| `q` | Quit |

Notes:
- `SIGKILL` requires confirmation in-app (`Enter` confirm, `Esc` cancel).
- Process control follows OS permissions; root-owned processes may return permission errors.

## Process Control

ramwise now supports direct process signaling from the process list:

1. Select a process with `j/k` or `↑/↓`.
2. Press `x` to send `SIGTERM` (graceful stop).
3. Press `X` to open kill confirmation, then:
   - `Enter` to send `SIGKILL`
   - `Esc` to cancel

Action results are shown as in-app status messages (success, warning, or error).

## Configuration

### Themes
ramwise includes built-in themes selectable via `--theme` or `-t`:
```bash
# Launch with dark theme (default)
ramwise -t dark

# Launch with light theme (Gruvbox Light)
ramwise -t light
```
You can also add a custom theme in $HOME/.config/ramwise/theme.toml. The theme in the app will be called custom and will not need to be registered by you.

Custom themes can be added to `src/ui/theme.rs` and registered in `App::new` (`src/app.rs`).

### Layout Customization
Layout dimensions and panel splits can be customized in themes


## Insight Rules

ramwise includes intelligent analysis rules:

| Rule | Severity | Description |
|------|----------|-------------|
| Memory Leak Detector | Warning/Critical | Detects consistent RSS growth patterns |
| Memory Hog | Warning | Flags processes using >30% of total RAM |
| Sudden Spike | Warning | Alerts on rapid memory increases (>100MB in 10s) |
| OOM Risk | Critical | Warns when system is at risk of OOM |
| Swap Pressure | Warning | Detects excessive swap usage |
| Fragmentation | Info | Identifies high VSS/RSS ratios |
| Cache Info | Info | Explains high page cache usage |

## Architecture

```
ramwise/
├── src/
│   ├── main.rs              # Entry point, async event loop
│   ├── app.rs               # Application state
│   ├── collector/           # Memory data collection from /proc
│   ├── analyzer/            # Rule engine and insights
│   ├── history/             # Time-series data buffer
│   ├── ui/                  # Ratatui TUI components
│   └── utils/               # Formatting utilities
```

## Requirements

- Linux kernel 2.6.28+ (for `/proc/[pid]/smaps_rollup`)
- Terminal with color support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Built with:
- [Ratatui](https://github.com/ratatui/ratatui) - TUI framework
- [procfs](https://github.com/eminence/procfs) - /proc filesystem parser
- [Tokio](https://tokio.rs/) - Async runtime
