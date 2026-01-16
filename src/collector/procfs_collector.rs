//! Procfs-based memory data collector

use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use procfs::process::all_processes;
use procfs::{Current, Meminfo};
use tokio::sync::mpsc;
use tokio::time::interval;

use super::types::{MemorySnapshot, ProcessMemory, SystemMemory};

/// Memory data collector that reads from /proc
pub struct Collector {
    /// Interval between collections
    interval: Duration,
    /// Whether to collect detailed smaps data (slower but more accurate)
    collect_smaps: bool,
    /// Minimum RSS to include a process (filter out tiny processes) - in bytes
    min_rss_bytes: u64,
}

impl Collector {
    /// Create a new collector with default settings
    pub fn new() -> Self {
        Self {
            interval: Duration::from_secs(1),
            collect_smaps: true,
            min_rss_bytes: 1024 * 1024, // 1 MB minimum
        }
    }

    /// Set collection interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Set whether to collect smaps data
    pub fn with_smaps(mut self, collect: bool) -> Self {
        self.collect_smaps = collect;
        self
    }

    /// Set minimum RSS threshold (in bytes)
    pub fn with_min_rss(mut self, min_bytes: u64) -> Self {
        self.min_rss_bytes = min_bytes;
        self
    }

    /// Collect a single memory snapshot
    pub fn collect_snapshot(&self) -> Result<MemorySnapshot> {
        let timestamp = Instant::now();

        // Collect system memory info
        let system = self.collect_system_memory()?;

        // Collect process memory info
        let (processes, total_processes, running_processes) = self.collect_processes()?;

        Ok(MemorySnapshot {
            timestamp,
            system,
            processes,
            total_processes,
            running_processes,
        })
    }

    /// Collect system-wide memory information from /proc/meminfo
    fn collect_system_memory(&self) -> Result<SystemMemory> {
        let meminfo = Meminfo::current().context("Failed to read /proc/meminfo")?;

        Ok(SystemMemory {
            total: meminfo.mem_total,
            available: meminfo.mem_available.unwrap_or(meminfo.mem_free),
            free: meminfo.mem_free,
            buffers: meminfo.buffers,
            cached: meminfo.cached,
            swap_total: meminfo.swap_total,
            swap_used: meminfo.swap_total.saturating_sub(meminfo.swap_free),
            slab: meminfo.slab,
            shared: meminfo.shmem.unwrap_or(0),
            active: meminfo.active,
            inactive: meminfo.inactive,
            dirty: meminfo.dirty,
            writeback: meminfo.writeback,
            mapped: meminfo.mapped,
        })
    }

    /// Collect memory information for all processes
    fn collect_processes(&self) -> Result<(Vec<ProcessMemory>, usize, usize)> {
        let mut processes = Vec::new();
        let mut total_count = 0;
        let mut running_count = 0;

        for proc_result in all_processes().context("Failed to enumerate processes")? {
            total_count += 1;

            let proc = match proc_result {
                Ok(p) => p,
                Err(_) => continue, // Process may have exited
            };

            // Get process status
            let status = match proc.status() {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Count running processes
            if status.state.starts_with('R') {
                running_count += 1;
            }

            // Get basic stats
            let stat = match proc.stat() {
                Ok(s) => s,
                Err(_) => continue,
            };

            // IMPORTANT: procfs crate returns VmRSS/VmSize in KILOBYTES, not bytes!
            // We need to convert kB -> bytes by multiplying by 1024
            let rss_bytes = status.vmrss.unwrap_or(0) * 1024;

            // Skip kernel threads (no virtual memory)
            let vss_bytes = status.vmsize.unwrap_or(0) * 1024;
            if vss_bytes == 0 {
                continue;
            }

            // Skip processes below threshold
            if rss_bytes > 0 && rss_bytes < self.min_rss_bytes {
                continue;
            }

            // Get command line
            let cmdline = proc
                .cmdline()
                .ok()
                .map(|v| v.join(" "))
                .unwrap_or_else(|| stat.comm.clone());

            // Get memory values from status (all in kB from procfs, convert to bytes)
            let shared = (status.rssfile.unwrap_or(0) + status.rssshmem.unwrap_or(0)) * 1024;
            let private = rss_bytes.saturating_sub(shared);
            let swap = status.vmswap.unwrap_or(0) * 1024;
            let heap = status.vmdata.unwrap_or(0) * 1024;
            let stack = status.vmstk.unwrap_or(0) * 1024;
            let libs = status.vmlib.unwrap_or(0) * 1024;

            let mut process = ProcessMemory {
                pid: proc.pid(),
                name: stat.comm.clone(),
                cmdline,
                state: status.state.chars().next().unwrap_or('?'),
                ppid: stat.ppid,
                uid: status.ruid,
                rss: rss_bytes,
                vss: vss_bytes,
                shared,
                private,
                swap,
                heap,
                stack,
                libs,
                minor_faults: stat.minflt,
                major_faults: stat.majflt,
                ..Default::default()
            };

            // Collect smaps_rollup for PSS/USS if enabled (requires read permission)
            if self.collect_smaps {
                if let Ok(smaps) = proc.smaps_rollup() {
                    // SmapsRollup contains a MemoryMaps which is Vec<MemoryMap>
                    if let Some(rollup) = smaps.memory_map_rollup.0.first() {
                        let ext = &rollup.extension.map;
                        
                        // Get values from the HashMap
                        process.pss = ext.get("Pss").copied().unwrap_or(0);
                        
                        let private_clean = ext.get("Private_Clean").copied().unwrap_or(0);
                        let private_dirty = ext.get("Private_Dirty").copied().unwrap_or(0);
                        process.uss = private_clean + private_dirty;
                        
                        process.anonymous = ext.get("Anonymous").copied().unwrap_or(0);

                        let shared_clean = ext.get("Shared_Clean").copied().unwrap_or(0);
                        let shared_dirty = ext.get("Shared_Dirty").copied().unwrap_or(0);
                        process.shared = shared_clean + shared_dirty;
                        process.private = private_clean + private_dirty;
                    }
                }
            }

            processes.push(process);
        }

        // Sort by RSS descending
        processes.sort_by(|a, b| b.rss.cmp(&a.rss));

        Ok((processes, total_count, running_count))
    }

    /// Run the collector as an async task, sending snapshots to a channel
    pub async fn run(self, tx: mpsc::Sender<MemorySnapshot>) -> Result<()> {
        let mut ticker = interval(self.interval);

        loop {
            ticker.tick().await;

            match self.collect_snapshot() {
                Ok(snapshot) => {
                    if tx.send(snapshot).await.is_err() {
                        // Receiver dropped, exit
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to collect snapshot: {}", e);
                }
            }
        }

        Ok(())
    }
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_snapshot() {
        let collector = Collector::new().with_smaps(false);
        let snapshot = collector.collect_snapshot().unwrap();

        assert!(snapshot.system.total > 0);
        // Should have at least the current process
        assert!(!snapshot.processes.is_empty(), "No processes found!");
        
        // Print some debug info
        eprintln!("Found {} processes", snapshot.processes.len());
        for p in snapshot.processes.iter().take(5) {
            eprintln!("  {} (PID {}): RSS={}", p.name, p.pid, p.rss);
        }
    }

    #[test]
    fn test_system_memory_calculations() {
        let mem = SystemMemory {
            total: 16 * 1024 * 1024 * 1024, // 16 GB
            available: 8 * 1024 * 1024 * 1024, // 8 GB
            ..Default::default()
        };

        assert_eq!(mem.used(), 8 * 1024 * 1024 * 1024);
        assert!((mem.usage_percent() - 50.0).abs() < 0.01);
    }
}
