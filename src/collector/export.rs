//! Versioned, portable snapshot contracts.
//!
//! This module intentionally does not serialize [`std::time::Instant`]. `Instant` is a
//! monotonic, process-local clock and has no meaning after an export is loaded elsewhere.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{MemoryRegion, MemorySnapshot, ProcessMemory, RegionMemory, SystemMemory};

/// Current wire-format version for exported snapshots.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Whether a collector feature was available for a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Available,
    Unavailable,
}

/// Metadata describing the collector that produced an export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectorMetadata {
    pub name: String,
    pub version: String,
    pub platform: String,
}

/// System metrics. All memory values are bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSystemMemory {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub free_bytes: u64,
    pub buffers_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub slab_bytes: u64,
    pub shared_bytes: u64,
    pub active_bytes: u64,
    pub inactive_bytes: u64,
    pub dirty_bytes: u64,
    pub writeback_bytes: u64,
    pub mapped_bytes: u64,
}

/// Detailed memory region. All memory values are bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportMemoryRegion {
    pub kind: Option<MemoryRegionKind>,
    /// May disclose private filesystem paths; callers should sanitize before sharing.
    pub path: Option<String>,
    pub size_bytes: u64,
    pub rss_bytes: u64,
    pub pss_bytes: u64,
    pub shared_clean_bytes: u64,
    pub shared_dirty_bytes: u64,
    pub private_clean_bytes: u64,
    pub private_dirty_bytes: u64,
}

/// Stable names for the runtime region classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRegionKind {
    Heap,
    Stack,
    Code,
    SharedLib,
    MappedFile,
    Anonymous,
    Vdso,
    Other,
}

/// Per-process metrics. All memory values are bytes; faults and identifiers are counts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProcessMemory {
    pub pid: i32,
    pub name: String,
    /// May contain sensitive command-line arguments; callers should sanitize before sharing.
    pub cmdline: String,
    pub state: char,
    pub ppid: i32,
    pub uid: u32,
    pub rss_bytes: u64,
    pub vss_bytes: u64,
    pub shared_bytes: u64,
    pub private_bytes: u64,
    pub pss_bytes: u64,
    pub uss_bytes: u64,
    pub swap_bytes: u64,
    pub heap_bytes: u64,
    pub stack_bytes: u64,
    pub libs_bytes: u64,
    pub anonymous_bytes: u64,
    pub file_mappings_bytes: u64,
    pub minor_faults: u64,
    pub major_faults: u64,
    pub regions: Option<Vec<ExportMemoryRegion>>,
}

/// Serializable snapshot contract. The timestamp is Unix milliseconds (wall-clock UTC).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSnapshot {
    pub schema_version: u32,
    pub captured_at_unix_ms: u64,
    pub collector: CollectorMetadata,
    pub capabilities: BTreeMap<String, Capability>,
    pub system: ExportSystemMemory,
    pub processes: Vec<ExportProcessMemory>,
    pub total_processes: usize,
    pub running_processes: usize,
}

/// Backwards-friendly name for consumers that refer to the wire model as a snapshot export.
#[allow(dead_code)]
pub type SnapshotExport = ExportSnapshot;

impl ExportSnapshot {
    /// Reject a payload from a schema version this binary does not understand.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version == SNAPSHOT_SCHEMA_VERSION {
            Ok(())
        } else {
            Err(format!(
                "unsupported snapshot schema version {}",
                self.schema_version
            ))
        }
    }

    /// Convert a runtime snapshot without exposing its process-local `Instant`.
    pub fn from_runtime(snapshot: &MemorySnapshot) -> Self {
        let mut capabilities = BTreeMap::from([
            ("processes".to_string(), Capability::Available),
            ("smaps_rollup".to_string(), Capability::Unavailable),
            ("regions".to_string(), Capability::Unavailable),
        ]);
        if snapshot.processes.iter().any(|p| p.pss != 0 || p.uss != 0) {
            capabilities.insert("smaps_rollup".to_string(), Capability::Available);
        }
        if snapshot.processes.iter().any(|p| p.regions.is_some()) {
            capabilities.insert("regions".to_string(), Capability::Available);
        }
        Self {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            captured_at_unix_ms: wall_clock_ms(),
            collector: CollectorMetadata {
                name: "ramwise-procfs".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                platform: std::env::consts::OS.into(),
            },
            capabilities,
            system: (&snapshot.system).into(),
            processes: snapshot.processes.iter().map(Into::into).collect(),
            total_processes: snapshot.total_processes,
            running_processes: snapshot.running_processes,
        }
    }

    /// Deterministic fixture for compatibility and serialization tests.
    pub fn fixture() -> Self {
        let mut fixture = Self::from_runtime(&MemorySnapshot::default());
        fixture.captured_at_unix_ms = 1_700_000_000_000;
        fixture.collector = CollectorMetadata {
            name: "ramwise-fixture".into(),
            version: "0.0.0".into(),
            platform: "test".into(),
        };
        fixture
            .capabilities
            .insert("smaps_rollup".into(), Capability::Unavailable);
        fixture
    }
}

fn wall_clock_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

macro_rules! map_fields {
    ($source:expr, $($field:ident => $export:ident),+ $(,)?) => {
        Self { $($export: $source.$field,)+ }
    };
}

impl From<&SystemMemory> for ExportSystemMemory {
    fn from(value: &SystemMemory) -> Self {
        map_fields!(value,
            total => total_bytes, available => available_bytes, free => free_bytes,
            buffers => buffers_bytes, cached => cached_bytes, swap_total => swap_total_bytes,
            swap_used => swap_used_bytes, slab => slab_bytes, shared => shared_bytes,
            active => active_bytes, inactive => inactive_bytes, dirty => dirty_bytes,
            writeback => writeback_bytes, mapped => mapped_bytes,
        )
    }
}

impl From<&ProcessMemory> for ExportProcessMemory {
    fn from(value: &ProcessMemory) -> Self {
        Self {
            pid: value.pid,
            name: value.name.clone(),
            cmdline: value.cmdline.clone(),
            state: value.state,
            ppid: value.ppid,
            uid: value.uid,
            rss_bytes: value.rss,
            vss_bytes: value.vss,
            shared_bytes: value.shared,
            private_bytes: value.private,
            pss_bytes: value.pss,
            uss_bytes: value.uss,
            swap_bytes: value.swap,
            heap_bytes: value.heap,
            stack_bytes: value.stack,
            libs_bytes: value.libs,
            anonymous_bytes: value.anonymous,
            file_mappings_bytes: value.file_mappings,
            minor_faults: value.minor_faults,
            major_faults: value.major_faults,
            regions: value
                .regions
                .as_ref()
                .map(|regions| regions.iter().map(Into::into).collect()),
        }
    }
}

impl From<&RegionMemory> for ExportMemoryRegion {
    fn from(value: &RegionMemory) -> Self {
        Self {
            kind: value.region_type.map(Into::into),
            path: value.path.clone(),
            size_bytes: value.size,
            rss_bytes: value.rss,
            pss_bytes: value.pss,
            shared_clean_bytes: value.shared_clean,
            shared_dirty_bytes: value.shared_dirty,
            private_clean_bytes: value.private_clean,
            private_dirty_bytes: value.private_dirty,
        }
    }
}

impl From<MemoryRegion> for MemoryRegionKind {
    fn from(value: MemoryRegion) -> Self {
        match value {
            MemoryRegion::Heap => Self::Heap,
            MemoryRegion::Stack => Self::Stack,
            MemoryRegion::Code => Self::Code,
            MemoryRegion::SharedLib => Self::SharedLib,
            MemoryRegion::MappedFile => Self::MappedFile,
            MemoryRegion::Anonymous => Self::Anonymous,
            MemoryRegion::Vdso => Self::Vdso,
            MemoryRegion::Other => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_round_trips_and_has_explicit_version() {
        let fixture = ExportSnapshot::fixture();
        let json = serde_json::to_string(&fixture).unwrap();
        let decoded: ExportSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, fixture);
        assert_eq!(decoded.schema_version, SNAPSHOT_SCHEMA_VERSION);
    }

    #[test]
    fn unavailable_capabilities_are_serialized_explicitly() {
        let fixture = ExportSnapshot::fixture();
        assert_eq!(
            fixture.capabilities["smaps_rollup"],
            Capability::Unavailable
        );
        let json = serde_json::to_string(&fixture).unwrap();
        assert!(json.contains("\"smaps_rollup\":\"unavailable\""));
    }

    #[test]
    fn unsupported_schema_versions_are_rejected() {
        let mut fixture = ExportSnapshot::fixture();
        fixture.schema_version += 1;
        assert!(fixture.validate().is_err());
    }

    #[test]
    fn fixture_is_deterministic() {
        assert_eq!(ExportSnapshot::fixture(), ExportSnapshot::fixture());
    }
}
