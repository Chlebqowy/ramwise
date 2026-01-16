//! Rule definitions for memory analysis

#![allow(dead_code)]

use std::time::Duration;

use crate::collector::MemorySnapshot;
use crate::history::HistoryBuffer;

use super::insights::{Insight, Severity};

/// Trait for analysis rules
pub trait Rule: Send + Sync {
    /// Unique name for this rule
    fn name(&self) -> &'static str;

    /// Evaluate the rule and return an insight if triggered
    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        history: &HistoryBuffer,
    ) -> Option<Insight>;
}

/// Detect potential memory leaks based on consistent growth
pub struct MemoryLeakDetector {
    /// Minimum growth percentage to trigger
    pub threshold_percent: f64,
    /// Duration to analyze
    pub duration: Duration,
    /// Minimum RSS to consider (ignore small processes)
    pub min_rss: u64,
}

impl Default for MemoryLeakDetector {
    fn default() -> Self {
        Self {
            threshold_percent: 20.0,
            duration: Duration::from_secs(180), // 3 minutes
            min_rss: 50 * 1024 * 1024, // 50 MB
        }
    }
}

impl Rule for MemoryLeakDetector {
    fn name(&self) -> &'static str {
        "memory_leak_detector"
    }

    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        history: &HistoryBuffer,
    ) -> Option<Insight> {
        for proc in &snapshot.processes {
            if proc.rss < self.min_rss {
                continue;
            }

            if let Some(stats) = history.growth_stats(proc.pid, self.duration) {
                if stats.percent_change >= self.threshold_percent
                    && history.is_consistently_growing(proc.pid, self.threshold_percent / 2.0)
                {
                    let rate_mb_per_min = (stats.rate_per_sec * 60.0) / (1024.0 * 1024.0);

                    return Some(
                        Insight::new(
                            format!("leak_{}_{}", proc.pid, proc.name),
                            if stats.percent_change > 50.0 {
                                Severity::Critical
                            } else {
                                Severity::Warning
                            },
                            format!(
                                "RSS grew {:.1}% in {:.0}s",
                                stats.percent_change,
                                stats.duration.as_secs_f64()
                            ),
                            format!(
                                "Memory increased from {} to {} ({:+.1} MB/min)",
                                format_bytes(stats.start_value),
                                format_bytes(stats.end_value),
                                rate_mb_per_min
                            ),
                            "Possible memory leak. Consider restarting or investigating allocations."
                                .to_string(),
                        )
                        .with_process(proc.pid, proc.insight_name()),
                    );
                }
            }
        }
        None
    }
}

/// Detect processes using excessive memory
pub struct MemoryHogDetector {
    /// Percentage of total RAM to trigger
    pub threshold_percent: f64,
}

impl Default for MemoryHogDetector {
    fn default() -> Self {
        Self {
            threshold_percent: 30.0,
        }
    }
}

impl Rule for MemoryHogDetector {
    fn name(&self) -> &'static str {
        "memory_hog_detector"
    }

    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        _history: &HistoryBuffer,
    ) -> Option<Insight> {
        let total = snapshot.system.total;
        if total == 0 {
            return None;
        }

        for proc in &snapshot.processes {
            let percent = (proc.rss as f64 / total as f64) * 100.0;
            if percent >= self.threshold_percent {
                return Some(
                    Insight::new(
                        format!("hog_{}_{}", proc.pid, proc.name),
                        Severity::Warning,
                        format!("Using {:.1}% of system RAM", percent),
                        format!(
                            "This process is using {} of {} total RAM",
                            format_bytes(proc.rss),
                            format_bytes(total)
                        ),
                        "Consider memory limits or alternative applications.".to_string(),
                    )
                    .with_process(proc.pid, proc.insight_name()),
                );
            }
        }
        None
    }
}

/// Detect sudden memory spikes
pub struct SuddenSpikeDetector {
    /// Minimum spike size in bytes
    pub min_spike_bytes: u64,
    /// Time window to check
    pub window: Duration,
}

impl Default for SuddenSpikeDetector {
    fn default() -> Self {
        Self {
            min_spike_bytes: 100 * 1024 * 1024, // 100 MB
            window: Duration::from_secs(10),
        }
    }
}

impl Rule for SuddenSpikeDetector {
    fn name(&self) -> &'static str {
        "sudden_spike_detector"
    }

    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        history: &HistoryBuffer,
    ) -> Option<Insight> {
        for proc in &snapshot.processes {
            if let Some(stats) = history.growth_stats(proc.pid, self.window) {
                let growth = stats.end_value.saturating_sub(stats.start_value);
                if growth >= self.min_spike_bytes {
                    return Some(
                        Insight::new(
                            format!("spike_{}_{}", proc.pid, proc.name),
                            Severity::Warning,
                            format!("Sudden +{} in {}s", format_bytes(growth), self.window.as_secs()),
                            format!(
                                "Memory jumped from {} to {} very quickly",
                                format_bytes(stats.start_value),
                                format_bytes(stats.end_value)
                            ),
                            "Check recent activity in this process.".to_string(),
                        )
                        .with_process(proc.pid, proc.insight_name()),
                    );
                }
            }
        }
        None
    }
}

/// Detect OOM risk
pub struct OomRiskDetector {
    /// Available memory threshold percentage
    pub available_threshold: f64,
    /// Swap usage threshold percentage
    pub swap_threshold: f64,
}

impl Default for OomRiskDetector {
    fn default() -> Self {
        Self {
            available_threshold: 5.0,
            swap_threshold: 80.0,
        }
    }
}

impl Rule for OomRiskDetector {
    fn name(&self) -> &'static str {
        "oom_risk_detector"
    }

    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        _history: &HistoryBuffer,
    ) -> Option<Insight> {
        let sys = &snapshot.system;
        let available_percent = if sys.total > 0 {
            (sys.available as f64 / sys.total as f64) * 100.0
        } else {
            100.0
        };

        let swap_percent = sys.swap_percent();

        if available_percent < self.available_threshold && swap_percent > self.swap_threshold {
            return Some(Insight::new(
                "oom_risk",
                Severity::Critical,
                format!(
                    "Low memory: {:.1}% available, {:.1}% swap used",
                    available_percent, swap_percent
                ),
                format!(
                    "Only {} available, {} swap used",
                    format_bytes(sys.available),
                    format_bytes(sys.swap_used)
                ),
                "System at risk of OOM. Close some applications immediately.".to_string(),
            ));
        }
        None
    }
}

/// Detect swap pressure
pub struct SwapPressureDetector {
    /// Swap usage threshold to trigger
    pub threshold_percent: f64,
}

impl Default for SwapPressureDetector {
    fn default() -> Self {
        Self {
            threshold_percent: 25.0,
        }
    }
}

impl Rule for SwapPressureDetector {
    fn name(&self) -> &'static str {
        "swap_pressure_detector"
    }

    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        _history: &HistoryBuffer,
    ) -> Option<Insight> {
        let sys = &snapshot.system;
        let swap_percent = sys.swap_percent();

        if swap_percent >= self.threshold_percent {
            return Some(Insight::new(
                "swap_pressure",
                Severity::Warning,
                format!("Swap usage at {:.1}%", swap_percent),
                format!(
                    "Using {} of {} swap space",
                    format_bytes(sys.swap_used),
                    format_bytes(sys.swap_total)
                ),
                "System is swapping, which may cause slowdowns.".to_string(),
            ));
        }
        None
    }
}

/// Detect high fragmentation
pub struct FragmentationDetector {
    /// VSS/RSS ratio threshold
    pub ratio_threshold: f64,
    /// Minimum RSS to consider
    pub min_rss: u64,
}

impl Default for FragmentationDetector {
    fn default() -> Self {
        Self {
            ratio_threshold: 10.0,
            min_rss: 100 * 1024 * 1024, // 100 MB
        }
    }
}

impl Rule for FragmentationDetector {
    fn name(&self) -> &'static str {
        "fragmentation_detector"
    }

    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        _history: &HistoryBuffer,
    ) -> Option<Insight> {
        for proc in &snapshot.processes {
            if proc.rss < self.min_rss {
                continue;
            }

            let ratio = proc.fragmentation_ratio();
            if ratio >= self.ratio_threshold {
                return Some(
                    Insight::new(
                        format!("frag_{}_{}", proc.pid, proc.name),
                        Severity::Info,
                        format!("High VSS/RSS ratio ({:.1}:1)", ratio),
                        format!(
                            "Virtual size {} vs actual {} ({:.1}x)",
                            format_bytes(proc.vss),
                            format_bytes(proc.rss),
                            ratio
                        ),
                        "Process has fragmented virtual address space.".to_string(),
                    )
                    .with_process(proc.pid, proc.insight_name()),
                );
            }
        }
        None
    }
}

/// Informational insight about page cache
pub struct CacheInfoRule;

impl Rule for CacheInfoRule {
    fn name(&self) -> &'static str {
        "cache_info"
    }

    fn evaluate(
        &self,
        snapshot: &MemorySnapshot,
        _history: &HistoryBuffer,
    ) -> Option<Insight> {
        let sys = &snapshot.system;
        let cache_percent = if sys.total > 0 {
            (sys.cached as f64 / sys.total as f64) * 100.0
        } else {
            0.0
        };

        // Only show if cache is significant (>40%)
        if cache_percent > 40.0 {
            return Some(Insight::new(
                "cache_info",
                Severity::Info,
                format!("Page cache using {:.1}% of RAM", cache_percent),
                format!(
                    "Kernel is caching {} of file data",
                    format_bytes(sys.cached)
                ),
                "This is normal and will be reclaimed when needed.".to_string(),
            ));
        }
        None
    }
}

/// Helper to format bytes as human-readable
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
