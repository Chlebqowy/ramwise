//! Procfs-based memory data collector

use anyhow::{Context, Result};
use sysinfo::System;
use tokio::sync::mpsc;
use tokio::time::interval;
use windows::Win32::System::SystemInformation::GlobalMemoryStatusEx;
use windows::Win32::System::SystemInformation::MEMORYSTATUSEX;
use windows_sys::Win32::System::ProcessStatus::GetPerformanceInfo;

use super::types::{MemoryRegion, MemorySnapshot, ProcessMemory, RegionMemory, SystemMemory};

/// Memory data collector that reads from /proc
pub struct Collector {
    /// Interval between collections
    interval: Duration,
    /// Minimum RSS to include a process (filter out tiny processes) - in bytes
    min_rss_bytes: u64,
}

impl Collector {
    /// Create a new collector with default settings
    pub fn new() -> Self {
        Self {
            interval: Duration::from_secs(1),
            min_rss_bytes: 1024 * 1024, // 1 MB minimum,
        }
    }

    /// Set collection interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
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
    fn collect_system_memory(&mut self) -> Result<SystemMemory> {
        let mut mem_status = MEMORYSTATUSEX::default();
        let mut perf_info = PERFORMANCE_INFORMATION::default();
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        // could maybe be in the struct but needs to be mut

        unsafe {
            GlobalMemoryStatusEx(&mut mem_status);
            GetPerformanceInfo(
                &mut perf_info,
                std::mem::size_of::<PERFORMANCE_INFORMATION>(),
            );
        } // needs to be ran each time

        let pagefiled_mem_total = mem_status.ullTotalPageFile;
        let pagefiled_mem_free = mem_status.ullAvailPageFile;
        let page_size = perf_info.PageSize;

        let (swap_in_pages, swap_out_pages, swap_in_rate, swap_out_rate) =
            self.track_paging(Instant::now());

        Ok(SystemMemory {
            total: mem_status.ullTotalPhys,
            available: mem_status.ullAvailPhys,
            cached: perf_info.SystemCache * page_size, // _PERFORMANCE_INFORMATION.SystemCache
            swap_total: pagefiled_mem_total,
            swap_used: mem_status
                .pagefiled_mem_total
                .saturating_sub(pagefiled_mem_free),
            swap_in_pages: 0,
            swap_out_pages: 0, // can't find, but task man sees it so it definitely exists
            windows_kernel_pool: perf_info.KernelTotal * page_size,
            // add standby memory, maybe warn if low in the future. lost the api though

            // TODO add insight rules for windows stuff, don't apply linux rules unless fields are present
            // TODO modify detail_panel
        })
    }
    /// Collect memory information for all processes
    // TODO use windows options for this instead, use different sames unless identical to linux. maybe even make this struct different
    fn collect_processes(&self) -> Result<(Vec<ProcessMemory>, usize, usize)> {
        let s = System::new_all();
        let mut processes = Vec::new();
        let mut total_count = 0;
        let mut running_count = None;

        for proc_result in s.processes() {
            total_count += 1;

            let proc = match proc_result {
                Ok(p) => p,
                Err(_) => continue, // Process may have exited
            };
            let rss_bytes = proc.memory(); //u64

            let vss_bytes = proc.virtual_memory; //u64

            // Skip processes below threshold
            if rss_bytes > 0 && rss_bytes < self.min_rss_bytes {
                continue;
            }

            // Get command line
            let cmdline = proc
                .exe()
                .unwrap_or(Path::new("not_found"))
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(); //.cmd() needs admin
            if cmdline == "not_found" {
                cmdline = proc.name()
            }

            // Get memory values from status (all in kB from procfs, convert to bytes)
            let shared = kib_to_bytes(status.rssfile.unwrap_or(0) + status.rssshmem.unwrap_or(0));
            let private = rss_bytes.saturating_sub(shared);
            let swap = kib_to_bytes(status.vmswap.unwrap_or(0));

            let pid = proc.pid();

            let mut process = ProcessMemory {
                pid, // turns out windows has pids https://learn.microsoft.com/en-us/windows/win32/procthread/process-handles-and-identifiers
                name: proc.name(),
                cmdline,
                state: None,
                ppid: proc.parent,
                // uid unavailable, make it Option<>
                rss: rss_bytes,
                vss: vss_bytes, // rename to real and virtual for windows
                shared,         // SharedCommitUsage;
                private,        // PrivateUsage;
                swap,           // PagefileUsage;
                //heap: None,    // unavailable
                //stack: None,   // unavailable
                //libs: None,    // unavailable
                faults, //   DWORD   PageFaultCount;https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex2
                        // make this comatible with linux version
                        // TODO get a handle and use GetProcessMemoryInfo
                        // pss and uss is used in linux, but windows doesn't have that. but everything that uses it will be different in windows anyways. maybe leave them out
            };
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
    } // seems fine
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

fn kib_to_bytes(value: u64) -> u64 {
    value.saturating_mul(1024)
}

// TODO adapt for windows
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
        let mut vmstat_file = std::fs::File::create(&vmstat).unwrap();
        writeln!(vmstat_file, "pswpin {pswpin}\npswpout {pswpout}\n").unwrap();
        (dir, vmstat)
    }

    fn cleanup(dir: &PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    // should work on windows
    #[test]
    fn kib_conversion_is_explicit_and_saturating() {
        assert_eq!(kib_to_bytes(0), 0);
        assert_eq!(kib_to_bytes(1), 1024);
        assert_eq!(kib_to_bytes(u64::MAX), u64::MAX);
    }

    // should work on windows
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
    // definitely broken on windows
    fn fixture_inputs_flow_into_the_snapshot() {
        let (dir, vmstat, pressure) = fixture_inputs("flow", 1200, 3400);
        let mut collector = Collector::new();
        let snapshot = collector.collect_snapshot().unwrap();
        assert_eq!(snapshot.system.swap_in_pages, 1200);
        assert_eq!(snapshot.system.swap_out_pages, 3400);
        assert_eq!(snapshot.system.swap_in_rate, None);
        cleanup(&dir);
    }

    #[test]
    // definitely broken on windows
    fn swap_tracking_computes_rates_without_meminfo() {
        use std::time::Duration;
        let (dir, vmstat, _) = fixture_inputs("rates", 1000, 2000);
        let mut collector = Collector::new();
        collector.vmstat_path = vmstat.clone();
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

    //  should work on windows
    #[test]
    fn missing_inputs_degrade_to_gaps_without_failing() {
        let missing = PathBuf::from("/nonexistent-ramwise-fixture/inputs");
        let mut collector = Collector::new();
        let snapshot = collector.collect_snapshot().unwrap();
        assert_eq!(snapshot.system.swap_in_pages, 0);
        assert_eq!(snapshot.system.swap_in_rate, None);
        assert!(!snapshot.system.pressure.is_available()); // valid even though it uses pressure
    }
}
