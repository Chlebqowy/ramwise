//! Procfs-based memory data collector

use anyhow::{Context, Result};
use procfs::process::all_processes;
use procfs::{Current, Meminfo};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

use super::system_inputs::{
    PRESSURE_MEMORY_PATH, TimedSample, VMSTAT_PATH, read_memory_pressure, read_vmstat_sample,
    swap_rates,
};
use super::types::{MemorySnapshot, ProcessMemory, SystemMemory};

/// Memory data collector that reads from /proc
pub struct Collector {
    /// Interval between collections
    interval: Duration,
    /// Whether to collect detailed smaps data (slower but more accurate)
    collect_smaps: bool,
    /// Minimum RSS to include a process (filter out tiny processes) - in bytes
    min_rss_bytes: u64,
    /// Previous vmstat reading for sample-to-sample swap rates; `None` until
    /// the first successful read so the first snapshot reports unknown rates
    prev_vmstat: Option<TimedSample>,
    /// vmstat input path (well-known `/proc/vmstat` live; fixture in tests)
    vmstat_path: PathBuf,
    /// memory-pressure input path (well-known `/proc/pressure/memory` live)
    pressure_path: PathBuf,
}

impl Collector {
    /// Create a new collector with default settings
    pub fn new() -> Self {
        Self {
            interval: Duration::from_secs(1),
            collect_smaps: true,
            min_rss_bytes: 1024 * 1024, // 1 MB minimum
            prev_vmstat: None,
            vmstat_path: PathBuf::from(VMSTAT_PATH),
            pressure_path: PathBuf::from(PRESSURE_MEMORY_PATH),
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
    pub fn collect_snapshot(&mut self) -> Result<MemorySnapshot> {
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

    /// Collect system-wide memory information from /proc/meminfo.
    ///
    /// `/proc/meminfo` is required: without it there is no snapshot. The
    /// vmstat and pressure inputs are best-effort instead — a missing file
    /// leaves cumulative counters at zero, rates at unknown, and pressure
    /// averages absent, which the export contract marks as explicit
    /// capability gaps.
    fn collect_system_memory(&mut self) -> Result<SystemMemory> {
        let meminfo = Meminfo::current().context("Failed to read /proc/meminfo")?;

        let (swap_in_pages, swap_out_pages, swap_in_rate, swap_out_rate) =
            self.update_swap_tracking(Instant::now());

        let pressure = read_memory_pressure(&self.pressure_path).unwrap_or_default();

        Ok(SystemMemory {
            total: meminfo.mem_total,
            available: meminfo.mem_available.unwrap_or(meminfo.mem_free),
            free: meminfo.mem_free,
            buffers: meminfo.buffers,
            cached: meminfo.cached,
            swap_total: meminfo.swap_total,
            swap_used: meminfo.swap_total.saturating_sub(meminfo.swap_free),
            swap_in_pages,
            swap_out_pages,
            swap_in_rate,
            swap_out_rate,
            slab: meminfo.slab,
            slab_reclaimable: meminfo.s_reclaimable.unwrap_or(0),
            slab_unreclaimable: meminfo.s_unreclaim.unwrap_or(0),
            kernel_stack: meminfo.kernel_stack.unwrap_or(0),
            pressure,
            shared: meminfo.shmem.unwrap_or(0),
            active: meminfo.active,
            inactive: meminfo.inactive,
            dirty: meminfo.dirty,
            writeback: meminfo.writeback,
            mapped: meminfo.mapped,
        })
    }

    /// Advance vmstat tracking: read the current counters, derive rates
    /// against the previous reading, and store the new baseline.
    /// Separated from meminfo so tests drive it with fixture paths and
    /// explicit timestamps, without depending on live /proc/meminfo.
    fn update_swap_tracking(&mut self, now: Instant) -> (u64, u64, Option<f64>, Option<f64>) {
        let current = read_vmstat_sample(&self.vmstat_path)
            .ok()
            .map(|sample| TimedSample { sample, at: now });
        let rates = match (&self.prev_vmstat, &current) {
            (Some(previous), Some(current)) => swap_rates(previous, current),
            _ => None,
        };
        self.prev_vmstat = current;
        match &self.prev_vmstat {
            Some(timed) => (
                timed.sample.pswpin,
                timed.sample.pswpout,
                rates.map(|rates| rates.in_per_sec),
                rates.map(|rates| rates.out_per_sec),
            ),
            None => (0, 0, None, None),
        }
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
            let rss_bytes = kib_to_bytes(status.vmrss.unwrap_or(0));

            // Skip kernel threads (no virtual memory)
            let vss_bytes = kib_to_bytes(status.vmsize.unwrap_or(0));
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
            let shared = kib_to_bytes(status.rssfile.unwrap_or(0) + status.rssshmem.unwrap_or(0));
            let private = rss_bytes.saturating_sub(shared);
            let swap = kib_to_bytes(status.vmswap.unwrap_or(0));
            let heap = kib_to_bytes(status.vmdata.unwrap_or(0));
            let stack = kib_to_bytes(status.vmstk.unwrap_or(0));
            let libs = kib_to_bytes(status.vmlib.unwrap_or(0));

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
            if self.collect_smaps
                && let Ok(smaps) = proc.smaps_rollup()
            {
                // SmapsRollup contains a MemoryMaps which is Vec<MemoryMap>
                if let Some(rollup) = smaps.memory_map_rollup.0.first() {
                    let ext = &rollup.extension.map;

                    // Get values from the HashMap
                    // procfs parses smaps values with their `kB` suffix into bytes.
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

            processes.push(process);
        }

        // Sort by RSS descending
        processes.sort_by_key(|a| std::cmp::Reverse(a.rss));

        Ok((processes, total_count, running_count))
    }

    /// Run the collector as an async task, sending snapshots to a channel
    pub async fn run(mut self, tx: mpsc::Sender<MemorySnapshot>) -> Result<()> {
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

fn kib_to_bytes(value: u64) -> u64 {
    value.saturating_mul(1024)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fixture_inputs(test: &str, pswpin: u64, pswpout: u64) -> (PathBuf, PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "ramwise-{}-{}-{}",
            test,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos())
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let vmstat = dir.join("vmstat");
        let pressure = dir.join("pressure");
        let mut vmstat_file = std::fs::File::create(&vmstat).unwrap();
        writeln!(vmstat_file, "pswpin {pswpin}\npswpout {pswpout}\n").unwrap();
        let mut pressure_file = std::fs::File::create(&pressure).unwrap();
        writeln!(
            pressure_file,
            "some avg10=1.25 avg60=0.50 avg300=0.10 total=1\nfull avg10=0.25 avg60=0.10 avg300=0.02 total=2\n"
        )
        .unwrap();
        (dir, vmstat, pressure)
    }

    fn cleanup(dir: &PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn kib_conversion_is_explicit_and_saturating() {
        assert_eq!(kib_to_bytes(0), 0);
        assert_eq!(kib_to_bytes(1), 1024);
        assert_eq!(kib_to_bytes(u64::MAX), u64::MAX);
    }

    #[test]
    fn test_system_memory_calculations() {
        let mem = SystemMemory {
            total: 16 * 1024 * 1024 * 1024,    // 16 GB
            available: 8 * 1024 * 1024 * 1024, // 8 GB
            ..Default::default()
        };

        assert_eq!(mem.used(), 8 * 1024 * 1024 * 1024);
        assert!((mem.usage_percent() - 50.0).abs() < 0.01);
        assert_eq!(mem.swap_percent(), 0.0);
    }

    #[test]
    fn fixture_inputs_flow_into_the_snapshot() {
        let (dir, vmstat, pressure) = fixture_inputs("flow", 1200, 3400);
        let mut collector = Collector::new();
        collector.vmstat_path = vmstat;
        collector.pressure_path = pressure;
        let snapshot = collector.collect_snapshot().unwrap();
        assert_eq!(snapshot.system.swap_in_pages, 1200);
        assert_eq!(snapshot.system.swap_out_pages, 3400);
        assert_eq!(snapshot.system.swap_in_rate, None);
        assert_eq!(snapshot.system.pressure.some_avg10, Some(1.25));
        assert_eq!(snapshot.system.pressure.full_avg300, Some(0.02));
        cleanup(&dir);
    }

    #[test]
    fn swap_tracking_computes_rates_without_meminfo() {
        use std::time::Duration;
        let (dir, vmstat, _) = fixture_inputs("rates", 1000, 2000);
        let mut collector = Collector::new();
        collector.vmstat_path = vmstat.clone();
        collector.pressure_path = PathBuf::from("/nonexistent-ramwise-fixture/pressure");
        let start = Instant::now();
        let (in_pages, out_pages, in_rate, out_rate) = collector.update_swap_tracking(start);
        assert_eq!((in_pages, out_pages), (1000, 2000));
        assert_eq!((in_rate, out_rate), (None, None));

        std::fs::write(&vmstat, "pswpin 1100\npswpout 2200\n").unwrap();
        let (_, _, in_rate, out_rate) =
            collector.update_swap_tracking(start + Duration::from_secs(10));
        assert_eq!(in_rate, Some(10.0));
        assert_eq!(out_rate, Some(20.0));
        cleanup(&dir);
    }

    #[test]
    fn missing_inputs_degrade_to_gaps_without_failing() {
        let missing = PathBuf::from("/nonexistent-ramwise-fixture/inputs");
        let mut collector = Collector::new();
        collector.vmstat_path = missing.join("vmstat");
        collector.pressure_path = missing.join("pressure");
        let snapshot = collector.collect_snapshot().unwrap();
        assert_eq!(snapshot.system.swap_in_pages, 0);
        assert_eq!(snapshot.system.swap_in_rate, None);
        assert!(!snapshot.system.pressure.is_available());
    }
}
