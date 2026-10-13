//! Memory data collection from /proc filesystem
//!
//! This module handles all data collection from the Linux kernel via procfs.
//! It runs as an async task, collecting memory snapshots at regular intervals.

mod export;
#[cfg(target_os = "linux")]
mod procfs_collector;

#[cfg(target_os = "windows")]
mod winapi_collector;

mod system_inputs;
mod types;
mod writers;

#[allow(unused_imports)]
pub use export::{
    Capability, CollectorMetadata, ExportMemoryRegion, ExportProcessMemory, ExportSnapshot,
    ExportSystemMemory, MemoryRegionKind, SNAPSHOT_SCHEMA_VERSION, SnapshotExport,
};
#[cfg(target_os = "linux")]
pub use procfs_collector::Collector;
#[cfg(target_os = "windows")]
pub use winapi_collector::Collector;
pub use types::{MemorySnapshot, ProcessMemory, SystemMemory};
pub use writers::{render_csv, render_json, write_target, write_targets};
