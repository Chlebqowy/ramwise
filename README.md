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

## Screenshots

![Screenshot](Screenshot.png)

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

### Arch Linux (AUR)

```bash
# Coming soon
yay -S ramwise
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
```

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

Sadly, there is no one single easy-to-use config. You have to make changes to the code itself but it's fairly easy and a lot is explained in comments. 

To modify the layout, go to src/ui/layout.rs. If you are not very experienced with coding, just modify the values in pub fn new() -> Self { Self {. If you can code, you can also modify the rest of the file. The layout and library is fairly simple. 

To modify the theme, go to src/ui/theme.rs
pub fn dark() -> Self {
        Self {
You can also copy that object and rename it to use another theme.
You can set a theme with --theme.
If the theme is custom, you have to add it to
impl App {
    /// Create a new application
    pub fn new(theme: str) -> Self {
in app.rs 


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
