//! Core data types for memory information

#![allow(dead_code)]

use std::time::Instant;

/// System-wide memory information from /proc/meminfo
#[derive(Debug, Clone, Default)]
pub struct SystemMemory {
    /// Total physical RAM in bytes
    pub total: u64,
    /// Memory available for starting new applications
    pub available: u64,
    /// Completely unused memory
    pub free: u64,
    /// Memory used for disk block buffers
    pub buffers: u64,
    /// Memory used for page cache (file contents)
    pub cached: u64,
    /// Total swap space
    pub swap_total: u64,
    /// Used swap space
    pub swap_used: u64,
    /// Kernel slab allocator memory
    pub slab: u64,
    /// Shared memory (shmem, tmpfs)
    pub shared: u64,
    /// Memory actively being used
    pub active: u64,
    /// Memory not recently used (candidate for reclaim)
    pub inactive: u64,
    /// Dirty pages waiting to be written to disk
    pub dirty: u64,
    /// Memory being actively written to disk
    pub writeback: u64,
    /// Memory mapped into page tables
    pub mapped: u64,
}

impl SystemMemory {
    /// Calculate used memory (total - available)
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    /// Calculate memory usage percentage
    pub fn usage_percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.used() as f64 / self.total as f64) * 100.0
    }

    /// Calculate swap usage percentage
    pub fn swap_percent(&self) -> f64 {
        if self.swap_total == 0 {
            return 0.0;
        }
        (self.swap_used as f64 / self.swap_total as f64) * 100.0
    }
}

/// Memory region types from /proc/[pid]/maps
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryRegion {
    Heap,
    Stack,
    Code,
    SharedLib,
    MappedFile,
    Anonymous,
    Vdso,
    Other,
}

/// Detailed memory breakdown for a region
#[derive(Debug, Clone, Default)]
pub struct RegionMemory {
    pub region_type: Option<MemoryRegion>,
    pub path: Option<String>,
    pub size: u64,
    pub rss: u64,
    pub pss: u64,
    pub shared_clean: u64,
    pub shared_dirty: u64,
    pub private_clean: u64,
    pub private_dirty: u64,
}

/// Per-process memory information
#[derive(Debug, Clone)]
pub struct ProcessMemory {
    /// Process ID
    pub pid: i32,
    /// Process name (comm)
    pub name: String,
    /// Full command line
    pub cmdline: String,
    /// Process state (R, S, D, Z, T, etc.)
    pub state: char,
    /// Parent process ID
    pub ppid: i32,
    /// User ID
    pub uid: u32,

    // === Basic memory stats ===
    /// Resident Set Size - actual physical memory used
    pub rss: u64,
    /// Virtual Size - total virtual address space
    pub vss: u64,
    /// Shared memory with other processes
    pub shared: u64,
    /// Private memory (not shared)
    pub private: u64,

    // === Advanced stats (from smaps_rollup) ===
    /// Proportional Set Size - shared memory divided proportionally
    pub pss: u64,
    /// Unique Set Size - memory unique to this process
    pub uss: u64,
    /// Swap usage
    pub swap: u64,

    // === Memory breakdown by type ===
    /// Heap memory
    pub heap: u64,
    /// Stack memory
    pub stack: u64,
    /// Memory-mapped libraries
    pub libs: u64,
    /// Anonymous mappings (malloc, etc.)
    pub anonymous: u64,
    /// Memory-mapped files
    pub file_mappings: u64,

    // === Page fault stats ===
    /// Minor page faults (no disk I/O)
    pub minor_faults: u64,
    /// Major page faults (disk I/O required)
    pub major_faults: u64,

    // === Detailed regions (optional, expensive to collect) ===
    pub regions: Option<Vec<RegionMemory>>,
}

impl Default for ProcessMemory {
    fn default() -> Self {
        Self {
            pid: 0,
            name: String::new(),
            cmdline: String::new(),
            state: '?',
            ppid: 0,
            uid: 0,
            rss: 0,
            vss: 0,
            shared: 0,
            private: 0,
            pss: 0,
            uss: 0,
            swap: 0,
            heap: 0,
            stack: 0,
            libs: 0,
            anonymous: 0,
            file_mappings: 0,
            minor_faults: 0,
            major_faults: 0,
            regions: None,
        }
    }
}

impl ProcessMemory {
    /// Get display name (truncated if necessary)
    pub fn display_name(&self, max_len: usize) -> String {
        if self.name.len() <= max_len {
            self.name.clone()
        } else {
            format!("{}...", &self.name[..max_len.saturating_sub(3)])
        }
    }

    /// Get a nice display name for insights - extracts the base binary name from cmdline
    /// Returns something like "zen-browser" instead of "zen-bin" or "/opt/zen-browser-bin/zen-bin"
    pub fn insight_name(&self) -> String {
        // First, try to get a meaningful name from cmdline
        if !self.cmdline.is_empty() && self.cmdline != self.name {
            // Get the first argument (the executable path)
            let first_arg = self.cmdline.split_whitespace().next().unwrap_or(&self.cmdline);
            // Get the base name from the path
            let base = first_arg.rsplit('/').next().unwrap_or(first_arg);
            // Return it if it's meaningful
            if !base.is_empty() && base.len() <= 30 {
                return base.to_string();
            }
        }
        // Fall back to the comm name
        self.name.clone()
    }

    /// Calculate fragmentation ratio (VSS/RSS)
    /// Higher values indicate more virtual address space fragmentation
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.rss == 0 {
            return 0.0;
        }
        self.vss as f64 / self.rss as f64
    }

    /// Check if this is a kernel thread
    pub fn is_kernel_thread(&self) -> bool {
        self.vss == 0 && self.rss == 0
    }
}

/// A complete memory snapshot at a point in time
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// When this snapshot was taken
    pub timestamp: Instant,
    /// System-wide memory information
    pub system: SystemMemory,
    /// Per-process memory information (sorted by RSS descending)
    pub processes: Vec<ProcessMemory>,
    /// Total number of processes (including kernel threads)
    pub total_processes: usize,
    /// Number of running processes
    pub running_processes: usize,
}

impl MemorySnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            timestamp: Instant::now(),
            system: SystemMemory::default(),
            processes: Vec::new(),
            total_processes: 0,
            running_processes: 0,
        }
    }

    /// Get top N processes by RSS
    pub fn top_by_rss(&self, n: usize) -> &[ProcessMemory] {
        &self.processes[..n.min(self.processes.len())]
    }

    /// Find a process by PID
    pub fn find_process(&self, pid: i32) -> Option<&ProcessMemory> {
        self.processes.iter().find(|p| p.pid == pid)
    }

    /// Calculate total user-space memory (sum of all process RSS)
    /// Note: This overcounts shared memory
    pub fn total_process_rss(&self) -> u64 {
        self.processes.iter().map(|p| p.rss).sum()
    }

    /// Calculate total PSS (more accurate, accounts for sharing)
    pub fn total_process_pss(&self) -> u64 {
        self.processes.iter().map(|p| p.pss).sum()
    }
}

impl Default for MemorySnapshot {
    fn default() -> Self {
        Self::new()
    }
}
