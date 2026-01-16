//! Memory data collection from /proc filesystem
//!
//! This module handles all data collection from the Linux kernel via procfs.
//! It runs as an async task, collecting memory snapshots at regular intervals.

mod procfs_collector;
mod types;

pub use procfs_collector::Collector;
pub use types::{MemorySnapshot, ProcessMemory, SystemMemory};
