//! Ring buffer for memory snapshot history

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::collector::{MemorySnapshot, SystemMemory};

/// Data point for a process at a specific time
#[derive(Debug, Clone)]
pub struct ProcessDataPoint {
    pub timestamp: Instant,
    pub rss: u64,
    pub pss: u64,
    pub private: u64,
    pub swap: u64,
}

/// Trend direction for memory usage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Increasing,
    Decreasing,
    Stable,
}

/// Growth statistics for a process
#[derive(Debug, Clone)]
pub struct GrowthStats {
    /// Bytes per second growth rate
    pub rate_per_sec: f64,
    /// Percentage change over the period
    pub percent_change: f64,
    /// Trend direction
    pub trend: Trend,
    /// Duration of the measurement period
    pub duration: Duration,
    /// Starting value
    pub start_value: u64,
    /// Ending value
    pub end_value: u64,
}

/// Ring buffer for memory snapshot history
pub struct HistoryBuffer {
    /// System-level snapshots
    system_history: VecDeque<(Instant, SystemMemory)>,
    /// Per-process history (indexed by PID)
    process_history: HashMap<i32, VecDeque<ProcessDataPoint>>,
    /// Maximum number of snapshots to keep
    capacity: usize,
    /// Maximum age of snapshots
    max_age: Duration,
}

impl HistoryBuffer {
    /// Create a new history buffer
    /// 
    /// # Arguments
    /// * `capacity` - Maximum number of snapshots to keep
    /// * `max_age` - Maximum age of snapshots before pruning
    pub fn new(capacity: usize, max_age: Duration) -> Self {
        Self {
            system_history: VecDeque::with_capacity(capacity),
            process_history: HashMap::new(),
            capacity,
            max_age,
        }
    }

    /// Create with default settings (5 minutes of history at 1s intervals)
    pub fn default_5min() -> Self {
        Self::new(300, Duration::from_secs(300))
    }

    /// Add a new snapshot to the history
    pub fn push(&mut self, snapshot: &MemorySnapshot) {
        let timestamp = snapshot.timestamp;

        // Add system data
        self.system_history
            .push_back((timestamp, snapshot.system.clone()));

        // Add process data
        for proc in &snapshot.processes {
            let entry = self
                .process_history
                .entry(proc.pid)
                .or_insert_with(|| VecDeque::with_capacity(self.capacity));

            entry.push_back(ProcessDataPoint {
                timestamp,
                rss: proc.rss,
                pss: proc.pss,
                private: proc.private,
                swap: proc.swap,
            });

            // Trim old entries for this process
            while entry.len() > self.capacity {
                entry.pop_front();
            }
        }

        // Trim old system entries
        while self.system_history.len() > self.capacity {
            self.system_history.pop_front();
        }

        // Prune old data
        self.prune_old_data(timestamp);
    }

    /// Remove data older than max_age
    fn prune_old_data(&mut self, now: Instant) {
        let cutoff = now - self.max_age;

        // Prune system history
        while let Some((ts, _)) = self.system_history.front() {
            if *ts < cutoff {
                self.system_history.pop_front();
            } else {
                break;
            }
        }

        // Prune process history and remove empty entries
        let mut to_remove = Vec::new();
        for (pid, history) in self.process_history.iter_mut() {
            while let Some(point) = history.front() {
                if point.timestamp < cutoff {
                    history.pop_front();
                } else {
                    break;
                }
            }
            if history.is_empty() {
                to_remove.push(*pid);
            }
        }
        for pid in to_remove {
            self.process_history.remove(&pid);
        }
    }

    /// Get system memory trend data for graphing
    pub fn system_trend(&self) -> Vec<(Instant, u64)> {
        self.system_history
            .iter()
            .map(|(ts, mem)| (*ts, mem.used()))
            .collect()
    }

    /// Get system memory trend as percentage
    pub fn system_trend_percent(&self) -> Vec<(Instant, f64)> {
        self.system_history
            .iter()
            .map(|(ts, mem)| (*ts, mem.usage_percent()))
            .collect()
    }

    /// Get process RSS trend data for graphing
    pub fn process_trend(&self, pid: i32) -> Vec<(Instant, u64)> {
        self.process_history
            .get(&pid)
            .map(|history| history.iter().map(|p| (p.timestamp, p.rss)).collect())
            .unwrap_or_default()
    }

    /// Get process RSS trend as normalized values (0.0 - 1.0)
    pub fn process_trend_normalized(&self, pid: i32) -> Vec<f64> {
        let trend = self.process_trend(pid);
        if trend.is_empty() {
            return Vec::new();
        }

        let max = trend.iter().map(|(_, v)| *v).max().unwrap_or(1) as f64;
        let min = trend.iter().map(|(_, v)| *v).min().unwrap_or(0) as f64;
        let range = max - min;

        if range < 1.0 {
            // No significant change, return flat line at 0.5
            return vec![0.5; trend.len()];
        }

        trend
            .iter()
            .map(|(_, v)| (*v as f64 - min) / range)
            .collect()
    }

    /// Calculate growth rate for a process over a duration
    pub fn growth_stats(&self, pid: i32, duration: Duration) -> Option<GrowthStats> {
        let history = self.process_history.get(&pid)?;
        if history.len() < 2 {
            return None;
        }

        let now = history.back()?.timestamp;
        let cutoff = now - duration;

        // Find the oldest point within the duration
        let start_point = history
            .iter()
            .find(|p| p.timestamp >= cutoff)?;
        let end_point = history.back()?;

        let actual_duration = end_point.timestamp.duration_since(start_point.timestamp);
        if actual_duration.as_secs_f64() < 1.0 {
            return None;
        }

        let start_value = start_point.rss;
        let end_value = end_point.rss;
        let diff = end_value as i64 - start_value as i64;

        let rate_per_sec = diff as f64 / actual_duration.as_secs_f64();
        let percent_change = if start_value > 0 {
            (diff as f64 / start_value as f64) * 100.0
        } else {
            0.0
        };

        // Determine trend (threshold: 1% change or 1MB)
        let trend = if percent_change > 1.0 || diff > 1_000_000 {
            Trend::Increasing
        } else if percent_change < -1.0 || diff < -1_000_000 {
            Trend::Decreasing
        } else {
            Trend::Stable
        };

        Some(GrowthStats {
            rate_per_sec,
            percent_change,
            trend,
            duration: actual_duration,
            start_value,
            end_value,
        })
    }

    /// Check if a process has consistent growth (potential leak)
    pub fn is_consistently_growing(&self, pid: i32, threshold_percent: f64) -> bool {
        let history = match self.process_history.get(&pid) {
            Some(h) if h.len() >= 10 => h,
            _ => return false,
        };

        // Check if most recent values are higher than earlier values
        let len = history.len();
        let first_quarter: Vec<_> = history.iter().take(len / 4).collect();
        let last_quarter: Vec<_> = history.iter().skip(3 * len / 4).collect();

        if first_quarter.is_empty() || last_quarter.is_empty() {
            return false;
        }

        let first_avg: f64 =
            first_quarter.iter().map(|p| p.rss as f64).sum::<f64>() / first_quarter.len() as f64;
        let last_avg: f64 =
            last_quarter.iter().map(|p| p.rss as f64).sum::<f64>() / last_quarter.len() as f64;

        if first_avg < 1.0 {
            return false;
        }

        let growth_percent = ((last_avg - first_avg) / first_avg) * 100.0;
        growth_percent > threshold_percent
    }

    /// Get the number of snapshots stored
    pub fn len(&self) -> usize {
        self.system_history.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.system_history.is_empty()
    }

    /// Get the number of tracked processes
    pub fn tracked_processes(&self) -> usize {
        self.process_history.len()
    }

    /// Get latest system memory
    pub fn latest_system(&self) -> Option<&SystemMemory> {
        self.system_history.back().map(|(_, mem)| mem)
    }

    /// Get latest RSS for a process
    pub fn latest_rss(&self, pid: i32) -> Option<u64> {
        self.process_history
            .get(&pid)?
            .back()
            .map(|p| p.rss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::ProcessMemory;

    #[test]
    fn test_history_buffer() {
        let mut buffer = HistoryBuffer::new(10, Duration::from_secs(60));

        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            system: SystemMemory {
                total: 16_000_000_000,
                available: 8_000_000_000,
                ..Default::default()
            },
            processes: vec![ProcessMemory {
                pid: 1234,
                name: "test".to_string(),
                rss: 100_000_000,
                ..Default::default()
            }],
            total_processes: 1,
            running_processes: 1,
        };

        buffer.push(&snapshot);

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.tracked_processes(), 1);
        assert!(buffer.latest_rss(1234).is_some());
    }

    #[test]
    fn test_growth_detection() {
        let mut buffer = HistoryBuffer::new(100, Duration::from_secs(60));

        // Simulate growing process
        let base_time = Instant::now();
        for i in 0..20 {
            let snapshot = MemorySnapshot {
                timestamp: base_time + Duration::from_secs(i),
                system: SystemMemory::default(),
                processes: vec![ProcessMemory {
                    pid: 1234,
                    name: "leaky".to_string(),
                    rss: 100_000_000 + (i as u64 * 10_000_000), // Growing by 10MB/sec
                    ..Default::default()
                }],
                total_processes: 1,
                running_processes: 1,
            };
            buffer.push(&snapshot);
        }

        assert!(buffer.is_consistently_growing(1234, 10.0));
    }
}
