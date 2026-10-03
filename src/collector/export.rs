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
///
/// Swap rates are pages per second; pressure averages are PSI percentages.
/// Optional values are `None` exactly when the input was unavailable, so a
/// missing `/proc` file serializes as an explicit `null` next to an
/// `unavailable` capability instead of a misleading zero.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExportSystemMemory {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub free_bytes: u64,
    pub buffers_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    /// Added after schema 1 shipped: defaulted on read so older files parse.
    #[serde(default)]
    pub swap_in_pages: u64,
    #[serde(default)]
    pub swap_out_pages: u64,
    pub swap_in_rate_per_sec: Option<f64>,
    pub swap_out_rate_per_sec: Option<f64>,
    pub slab_bytes: u64,
    /// Added after schema 1 shipped: defaulted on read so older files parse.
    #[serde(default)]
    pub slab_reclaimable_bytes: u64,
    #[serde(default)]
    pub slab_unreclaimable_bytes: u64,
    #[serde(default)]
    pub kernel_stack_bytes: u64,
    pub pressure_some_avg10: Option<f32>,
    pub pressure_some_avg60: Option<f32>,
    pub pressure_some_avg300: Option<f32>,
    pub pressure_full_avg10: Option<f32>,
    pub pressure_full_avg60: Option<f32>,
    pub pressure_full_avg300: Option<f32>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Reject a payload from a schema version this binary does not understand,
    /// or one carrying non-finite floats (rates and pressure must be real
    /// numbers or explicit nulls, never NaN or infinity).
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SNAPSHOT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported snapshot schema version {}",
                self.schema_version
            ));
        }
        let system = &self.system;
        let finite = [system.swap_in_rate_per_sec, system.swap_out_rate_per_sec]
            .into_iter()
            .flatten()
            .all(f64::is_finite);
        let pressures = [
            system.pressure_some_avg10,
            system.pressure_some_avg60,
            system.pressure_some_avg300,
            system.pressure_full_avg10,
            system.pressure_full_avg60,
            system.pressure_full_avg300,
        ]
        .into_iter()
        .flatten()
        .all(f32::is_finite);
        if finite && pressures {
            Ok(())
        } else {
            Err("non-finite swap rate or pressure value".to_string())
        }
    }

    /// Convert a runtime snapshot without exposing its process-local `Instant`.
    pub fn from_runtime(snapshot: &MemorySnapshot) -> Self {
        let mut capabilities = BTreeMap::from([
            ("processes".to_string(), Capability::Available),
            ("smaps_rollup".to_string(), Capability::Unavailable),
            ("regions".to_string(), Capability::Unavailable),
            ("swap_rates".to_string(), Capability::Unavailable),
            ("pressure".to_string(), Capability::Unavailable),
        ]);
        if snapshot.processes.iter().any(|p| p.pss != 0 || p.uss != 0) {
            capabilities.insert("smaps_rollup".to_string(), Capability::Available);
        }
        if snapshot.processes.iter().any(|p| p.regions.is_some()) {
            capabilities.insert("regions".to_string(), Capability::Available);
        }
        if snapshot.system.swap_in_rate.is_some() && snapshot.system.swap_out_rate.is_some() {
            capabilities.insert("swap_rates".to_string(), Capability::Available);
        }
        if snapshot.system.pressure.is_available() {
            capabilities.insert("pressure".to_string(), Capability::Available);
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

impl From<&SystemMemory> for ExportSystemMemory {
    fn from(value: &SystemMemory) -> Self {
        Self {
            total_bytes: value.total,
            available_bytes: value.available,
            free_bytes: value.free,
            buffers_bytes: value.buffers,
            cached_bytes: value.cached,
            swap_total_bytes: value.swap_total,
            swap_used_bytes: value.swap_used,
            swap_in_pages: value.swap_in_pages,
            swap_out_pages: value.swap_out_pages,
            swap_in_rate_per_sec: value.swap_in_rate,
            swap_out_rate_per_sec: value.swap_out_rate,
            slab_bytes: value.slab,
            slab_reclaimable_bytes: value.slab_reclaimable,
            slab_unreclaimable_bytes: value.slab_unreclaimable,
            kernel_stack_bytes: value.kernel_stack,
            pressure_some_avg10: value.pressure.some_avg10,
            pressure_some_avg60: value.pressure.some_avg60,
            pressure_some_avg300: value.pressure.some_avg300,
            pressure_full_avg10: value.pressure.full_avg10,
            pressure_full_avg60: value.pressure.full_avg60,
            pressure_full_avg300: value.pressure.full_avg300,
            shared_bytes: value.shared,
            active_bytes: value.active,
            inactive_bytes: value.inactive,
            dirty_bytes: value.dirty,
            writeback_bytes: value.writeback,
            mapped_bytes: value.mapped,
        }
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
    fn non_finite_values_are_rejected() {
        let mut fixture = ExportSnapshot::fixture();
        fixture.system.swap_in_rate_per_sec = Some(f64::NAN);
        assert!(fixture.validate().is_err());
        fixture.system.swap_in_rate_per_sec = Some(1.0);
        fixture.system.pressure_some_avg10 = Some(f32::INFINITY);
        assert!(fixture.validate().is_err());
        fixture.system.pressure_some_avg10 = Some(1.0);
        assert!(fixture.validate().is_ok());
    }

    #[test]
    fn fixture_is_deterministic() {
        assert_eq!(ExportSnapshot::fixture(), ExportSnapshot::fixture());
    }

    #[test]
    fn runtime_conversion_preserves_bytes_and_region_capabilities() {
        let mut snapshot = MemorySnapshot::default();
        snapshot.system.total = 4096;
        snapshot.system.available = 1024;
        snapshot.processes = vec![ProcessMemory {
            pid: 7,
            name: "worker".into(),
            rss: 2048,
            pss: 1536,
            uss: 1024,
            regions: Some(vec![RegionMemory {
                region_type: Some(MemoryRegion::Heap),
                path: Some("[heap]".into()),
                size: 4096,
                rss: 2048,
                ..Default::default()
            }]),
            ..Default::default()
        }];

        let export = ExportSnapshot::from_runtime(&snapshot);
        assert_eq!(export.system.total_bytes, 4096);
        assert_eq!(export.processes[0].rss_bytes, 2048);
        assert_eq!(
            export.processes[0].regions.as_ref().unwrap()[0].kind,
            Some(MemoryRegionKind::Heap)
        );
        assert_eq!(export.capabilities["smaps_rollup"], Capability::Available);
        assert_eq!(export.capabilities["regions"], Capability::Available);
        assert!(export.validate().is_ok());
    }

    #[test]
    fn json_contract_has_stable_field_names() {
        let json = serde_json::to_value(ExportSnapshot::fixture()).unwrap();
        assert_eq!(json["schema_version"], SNAPSHOT_SCHEMA_VERSION);
        assert!(json["system"]["total_bytes"].is_number());
        assert!(json["processes"].is_array());
        assert_eq!(json["capabilities"]["processes"], "available");
    }

    #[test]
    fn unavailable_inputs_are_explicit_gaps_not_zeros() {
        let export = ExportSnapshot::from_runtime(&MemorySnapshot::default());
        assert_eq!(export.capabilities["swap_rates"], Capability::Unavailable);
        assert_eq!(export.capabilities["pressure"], Capability::Unavailable);
        assert_eq!(export.system.swap_in_rate_per_sec, None);
        assert_eq!(export.system.pressure_some_avg10, None);
        let json = serde_json::to_value(&export).unwrap();
        assert!(json["system"]["swap_in_rate_per_sec"].is_null());
        assert!(json["system"]["pressure_some_avg10"].is_null());
        assert_eq!(json["capabilities"]["swap_rates"], "unavailable");
        assert_eq!(json["capabilities"]["pressure"], "unavailable");
    }

    #[test]
    fn snapshots_predating_new_system_fields_still_parse() {
        let mut json = serde_json::to_value(ExportSnapshot::fixture()).unwrap();
        let system = json["system"].as_object_mut().unwrap();
        for field in [
            "swap_in_pages",
            "swap_out_pages",
            "swap_in_rate_per_sec",
            "swap_out_rate_per_sec",
            "slab_reclaimable_bytes",
            "slab_unreclaimable_bytes",
            "kernel_stack_bytes",
            "pressure_some_avg10",
            "pressure_full_avg300",
        ] {
            system.remove(field);
        }
        let decoded: ExportSnapshot = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.system.swap_in_pages, 0);
        assert_eq!(decoded.system.kernel_stack_bytes, 0);
        assert_eq!(decoded.system.pressure_full_avg300, None);
        assert!(decoded.validate().is_ok());
    }

    #[test]
    fn present_rates_and_pressure_mark_capabilities_available() {
        let mut snapshot = MemorySnapshot::default();
        snapshot.system.swap_in_pages = 1200;
        snapshot.system.swap_out_pages = 3400;
        snapshot.system.swap_in_rate = Some(10.0);
        snapshot.system.swap_out_rate = Some(20.0);
        snapshot.system.pressure.some_avg10 = Some(1.25);
        let export = ExportSnapshot::from_runtime(&snapshot);
        assert_eq!(export.capabilities["swap_rates"], Capability::Available);
        assert_eq!(export.capabilities["pressure"], Capability::Available);
        assert_eq!(export.system.swap_in_pages, 1200);
        assert_eq!(export.system.swap_in_rate_per_sec, Some(10.0));
        assert_eq!(export.system.pressure_some_avg10, Some(1.25));
        assert!(export.validate().is_ok());
    }
}
