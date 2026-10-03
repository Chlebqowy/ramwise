//! Memory data collection from /proc filesystem
//!
//! This module handles all data collection from the Linux kernel via procfs.
//! It runs as an async task, collecting memory snapshots at regular intervals.

mod export;
mod procfs_collector;
mod system_inputs;
mod types;

#[allow(unused_imports)]
pub use export::{
    Capability, CollectorMetadata, ExportMemoryRegion, ExportProcessMemory, ExportSnapshot,
    ExportSystemMemory, MemoryRegionKind, SNAPSHOT_SCHEMA_VERSION, SnapshotExport,
};
pub use procfs_collector::Collector;
pub use types::{MemorySnapshot, ProcessMemory, SystemMemory};
