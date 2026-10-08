use std::time::{Duration, Instant};

use crate::collector::{MemorySnapshot, ProcessMemory, SystemMemory};

pub const FIXTURE_PID: i32 = 4242;

pub fn system_memory() -> SystemMemory {
    SystemMemory {
        total: 16 * 1024 * 1024 * 1024,
        available: 8 * 1024 * 1024 * 1024,
        free: 2 * 1024 * 1024 * 1024,
        buffers: 256 * 1024 * 1024,
        cached: 3 * 1024 * 1024 * 1024,
        swap_total: 4 * 1024 * 1024 * 1024,
        swap_used: 512 * 1024 * 1024,
        ..Default::default()
    }
}

pub fn process(rss: u64) -> ProcessMemory {
    ProcessMemory {
        pid: FIXTURE_PID,
        name: "fixture-worker".into(),
        cmdline: "/usr/bin/fixture-worker --stable".into(),
        state: 'R',
        ppid: 1,
        uid: 1000,
        rss,
        vss: rss * 2,
        shared: rss / 4,
        private: rss * 3 / 4,
        pss: rss * 3 / 4,
        uss: rss / 2,
        heap: rss / 3,
        stack: rss / 16,
        libs: rss / 8,
        anonymous: rss / 2,
        file_mappings: rss / 8,
        minor_faults: 12,
        major_faults: 2,
        ..Default::default()
    }
}

pub fn snapshot_at(timestamp: Instant, rss: u64) -> MemorySnapshot {
    MemorySnapshot {
        timestamp,
        system: system_memory(),
        processes: vec![process(rss)],
        total_processes: 3,
        running_processes: 1,
    }
}

pub fn history_snapshots(count: usize, start_rss: u64, step: u64) -> Vec<MemorySnapshot> {
    let start = Instant::now();
    (0..count)
        .map(|index| {
            snapshot_at(
                start + Duration::from_secs(index as u64),
                start_rss + step * index as u64,
            )
        })
        .collect()
}
