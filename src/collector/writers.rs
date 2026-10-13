//! JSON and CSV snapshot writers.
//!
//! File layout contracts:
//!
//! - JSON: the [`ExportSnapshot`] contract serialized compactly. Round-trips
//!   through `serde_json` and [`ExportSnapshot::validate`].
//! - CSV: one `#` comment line (`ramwise-csv schema_version=N
//!   captured_at_unix_ms=M`), one `# system_<field>=<value>` comment per
//!   system metric, then a single stable process table. The column header is
//!   part of the contract: columns are only appended, and only together with
//!   the opt-in `--export-details` flag, which adds a trailing `regions_json`
//!   column. Without details, region data is omitted, never zeroed.
//!
//! Writers refuse to overwrite an existing file unless `force` is set, so an
//! export can never silently clobber a previous capture. `-` as a path means
//! stdout (used by `--once`-style pipelines); nothing else is created.

use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use super::export::ExportSnapshot;

/// Stable process-table columns. Append-only; see module docs.
const CSV_COLUMNS: &[&str] = &[
    "pid",
    "name",
    "state",
    "ppid",
    "uid",
    "rss_bytes",
    "vss_bytes",
    "shared_bytes",
    "private_bytes",
    "pss_bytes",
    "uss_bytes",
    "swap_bytes",
    "heap_bytes",
    "stack_bytes",
    "libs_bytes",
    "anonymous_bytes",
    "file_mappings_bytes",
    "minor_faults",
    "major_faults",
    "cmdline",
];

/// Serialize one snapshot to versioned JSON text.
pub fn render_json(snapshot: &ExportSnapshot) -> Result<String> {
    snapshot
        .validate()
        .map_err(|message| anyhow::anyhow!("{message}"))?;
    serde_json::to_string(snapshot).context("Failed to serialize snapshot to JSON")
}

/// Render one snapshot to the documented CSV contract.
pub fn render_csv(snapshot: &ExportSnapshot, include_details: bool) -> Result<String> {
    snapshot
        .validate()
        .map_err(|message| anyhow::anyhow!("{message}"))?;
    let system = &snapshot.system;
    let mut out = String::new();
    writeln!(
        out,
        "# ramwise-csv schema_version={} captured_at_unix_ms={}",
        snapshot.schema_version, snapshot.captured_at_unix_ms
    )
    .unwrap();
    for (field, value) in [
        ("collector_name", snapshot.collector.name.clone()),
        ("collector_version", snapshot.collector.version.clone()),
        ("collector_platform", snapshot.collector.platform.clone()),
        ("collector_host", snapshot.collector.host.clone()),
        ("total_processes", snapshot.total_processes.to_string()),
        ("running_processes", snapshot.running_processes.to_string()),
    ] {
        writeln!(out, "# {field}={value}").unwrap();
    }
    for (capability, status) in &snapshot.capabilities {
        writeln!(out, "# capability_{capability}={status:?}").unwrap();
    }
    for (field, value) in [
        ("total_bytes", serde_json::to_string(&system.total_bytes)?),
        (
            "available_bytes",
            serde_json::to_string(&system.available_bytes)?,
        ),
        ("free_bytes", serde_json::to_string(&system.free_bytes)?),
        (
            "buffers_bytes",
            serde_json::to_string(&system.buffers_bytes)?,
        ),
        ("cached_bytes", serde_json::to_string(&system.cached_bytes)?),
        (
            "swap_total_bytes",
            serde_json::to_string(&system.swap_total_bytes)?,
        ),
        (
            "swap_used_bytes",
            serde_json::to_string(&system.swap_used_bytes)?,
        ),
        (
            "swap_in_pages",
            serde_json::to_string(&system.swap_in_pages)?,
        ),
        (
            "swap_out_pages",
            serde_json::to_string(&system.swap_out_pages)?,
        ),
        (
            "swap_in_rate_per_sec",
            serde_json::to_string(&system.swap_in_rate_per_sec)?,
        ),
        (
            "swap_out_rate_per_sec",
            serde_json::to_string(&system.swap_out_rate_per_sec)?,
        ),
        ("slab_bytes", serde_json::to_string(&system.slab_bytes)?),
        (
            "slab_reclaimable_bytes",
            serde_json::to_string(&system.slab_reclaimable_bytes)?,
        ),
        (
            "slab_unreclaimable_bytes",
            serde_json::to_string(&system.slab_unreclaimable_bytes)?,
        ),
        (
            "kernel_stack_bytes",
            serde_json::to_string(&system.kernel_stack_bytes)?,
        ),
        (
            "pressure_some_avg10",
            serde_json::to_string(&system.pressure_some_avg10)?,
        ),
        (
            "pressure_some_avg60",
            serde_json::to_string(&system.pressure_some_avg60)?,
        ),
        (
            "pressure_some_avg300",
            serde_json::to_string(&system.pressure_some_avg300)?,
        ),
        (
            "pressure_full_avg10",
            serde_json::to_string(&system.pressure_full_avg10)?,
        ),
        (
            "pressure_full_avg60",
            serde_json::to_string(&system.pressure_full_avg60)?,
        ),
        (
            "pressure_full_avg300",
            serde_json::to_string(&system.pressure_full_avg300)?,
        ),
        ("shared_bytes", serde_json::to_string(&system.shared_bytes)?),
        ("active_bytes", serde_json::to_string(&system.active_bytes)?),
        (
            "inactive_bytes",
            serde_json::to_string(&system.inactive_bytes)?,
        ),
        ("dirty_bytes", serde_json::to_string(&system.dirty_bytes)?),
        (
            "writeback_bytes",
            serde_json::to_string(&system.writeback_bytes)?,
        ),
        ("mapped_bytes", serde_json::to_string(&system.mapped_bytes)?),
    ] {
        writeln!(out, "# system_{field}={value}").unwrap();
    }
    let mut header = CSV_COLUMNS.join(",");
    if include_details {
        header.push_str(",regions_json");
    }
    writeln!(out, "{header}").unwrap();
    for process in &snapshot.processes {
        let mut row = vec![
            process.pid.to_string(),
            escape_csv(&process.name),
            escape_csv(&process.state.to_string()),
            process.ppid.to_string(),
            process.uid.to_string(),
            process.rss_bytes.to_string(),
            process.vss_bytes.to_string(),
            process.shared_bytes.to_string(),
            process.private_bytes.to_string(),
            process.pss_bytes.to_string(),
            process.uss_bytes.to_string(),
            process.swap_bytes.to_string(),
            process.heap_bytes.to_string(),
            process.stack_bytes.to_string(),
            process.libs_bytes.to_string(),
            process.anonymous_bytes.to_string(),
            process.file_mappings_bytes.to_string(),
            process.minor_faults.to_string(),
            process.major_faults.to_string(),
            escape_csv(&process.cmdline),
        ];
        if include_details {
            let regions = match &process.regions {
                Some(regions) => serde_json::to_string(regions).with_context(|| {
                    format!("failed to serialize regions for pid {}", process.pid)
                })?,
                None => String::new(),
            };
            row.push(escape_csv(&regions));
        }
        writeln!(out, "{}", row.join(",")).expect("writing to a String cannot fail");
    }
    Ok(out)
}

/// Minimal CSV escaping: quote when the field contains a comma, quote,
/// carriage return or newline, doubling embedded quotes.
fn escape_csv(field: &str) -> String {
    if field.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Write `contents` to `path`, refusing to overwrite unless `force`.
/// A path of `-` writes to stdout instead of touching the filesystem.
///
/// Writes use a same-directory temporary file and a final link/rename, so
/// readers never observe a partial snapshot. Without `force`, the final link
/// is created atomically and fails when the destination already exists.
pub fn write_target(path: &Path, contents: &str, force: bool) -> Result<()> {
    if path.as_os_str() == "-" {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(contents.as_bytes())
            .context("failed to write to stdout")?;
        stdout.flush().context("failed to flush stdout")?;
        return Ok(());
    }
    let temp_path = stage_target(path, contents, "ramwise")?;

    let result = (|| -> Result<()> {
        if force {
            fs::rename(&temp_path, path)
                .with_context(|| format!("failed to replace {}", path.display()))?;
        } else {
            match fs::hard_link(&temp_path, path) {
                Ok(()) => fs::remove_file(&temp_path)
                    .with_context(|| format!("failed to remove {}", temp_path.display()))?,
                Err(_error) if path.exists() => {
                    anyhow::bail!("refusing to overwrite existing file {}", path.display())
                }
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("failed to create {}", path.display()));
                }
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn stage_target(path: &Path, contents: &str, suffix: &str) -> Result<std::path::PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .context("export path must name a file")?
        .to_string_lossy();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temp_path = parent.join(format!(
        ".{name}.{suffix}-{pid}-{nonce}.tmp",
        pid = std::process::id()
    ));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .with_context(|| format!("failed to open temporary export {}", temp_path.display()))?;
        file.write_all(contents.as_bytes())
            .with_context(|| format!("failed to write {}", path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync {}", path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = remove_if_exists(&temp_path);
    }
    result?;
    Ok(temp_path)
}

fn remove_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Write multiple filesystem targets as one transaction. Temporary files are
/// fully synced before any destination is changed; a failed commit restores
/// every destination already touched by this call.
pub fn write_targets(targets: &[(&Path, &str)], force: bool) -> Result<()> {
    if targets.is_empty() {
        return Ok(());
    }

    let mut staged: Vec<(std::path::PathBuf, &Path)> = Vec::with_capacity(targets.len());
    for (path, contents) in targets {
        let temp_path = match stage_target(path, contents, "ramwise-stage") {
            Ok(temp_path) => temp_path,
            Err(error) => {
                for (staged_path, _) in &staged {
                    let _ = fs::remove_file(staged_path);
                }
                return Err(error);
            }
        };
        if !temp_path.exists() {
            for (staged_path, _) in &staged {
                let _ = fs::remove_file(staged_path);
            }
            anyhow::bail!("staged export disappeared before commit");
        }
        staged.push((temp_path, *path));
    }

    let mut committed = Vec::with_capacity(staged.len());
    let mut backups: Vec<(std::path::PathBuf, std::path::PathBuf)> =
        Vec::with_capacity(staged.len());
    let commit_result = (|| -> Result<()> {
        for (temp_path, path) in &staged {
            let backup = if force && path.exists() {
                if !path.is_file() {
                    anyhow::bail!(
                        "refusing to replace non-file export target {}",
                        path.display()
                    );
                }
                let backup = path.with_file_name(format!(
                    ".{}.ramwise-backup-{}-{}-{}",
                    path.file_name().unwrap().to_string_lossy(),
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_or(0, |duration| duration.as_nanos()),
                    backups.len()
                ));
                fs::hard_link(path, &backup).with_context(|| {
                    format!("failed to create exclusive backup for {}", path.display())
                })?;
                backups.push((path.to_path_buf(), backup.clone()));
                Some(backup)
            } else {
                None
            };
            if force {
                if let Err(error) = fs::rename(temp_path, path) {
                    if let Some(backup) = backup {
                        let _ = restore_backup(path, &backup);
                    }
                    return Err(error)
                        .with_context(|| format!("failed to commit {}", path.display()));
                }
            } else {
                fs::hard_link(temp_path, path)
                    .with_context(|| format!("failed to create {}", path.display()))?;
            }
            committed.push((path.to_path_buf(), backup));
            if !force {
                fs::remove_file(temp_path)
                    .with_context(|| format!("failed to remove {}", temp_path.display()))?;
            }
        }
        Ok(())
    })();

    if let Err(error) = commit_result {
        let mut rollback_error = None;
        for (path, backup) in committed.into_iter().rev() {
            if let Err(rollback) = remove_if_exists(&path) {
                rollback_error = Some(anyhow::anyhow!(
                    "failed to remove committed {}: {rollback}",
                    path.display()
                ));
            }
            if let Some(backup) = backup
                && let Err(rollback) = restore_backup(&path, &backup)
            {
                rollback_error = Some(anyhow::anyhow!(
                    "failed to restore {}: {rollback}",
                    path.display()
                ));
            }
        }
        for (temp_path, _) in &staged {
            if let Err(rollback) = remove_if_exists(temp_path) {
                rollback_error = Some(anyhow::anyhow!(
                    "failed to remove staged export {}: {rollback}",
                    temp_path.display()
                ));
            }
        }
        if let Some(rollback_error) = rollback_error {
            return Err(error).context(format!(
                "coordinated export rollback failed: {rollback_error}"
            ));
        }

        return Err(error);
    }
    for (_, backup) in backups {
        fs::remove_file(&backup)
            .with_context(|| format!("failed to remove backup {}", backup.display()))?;
    }
    Ok(())
}

fn restore_backup(path: &Path, backup: &Path) -> io::Result<()> {
    fs::hard_link(backup, path)?;
    fs::remove_file(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::ExportProcessMemory;

    fn snapshot_with_process() -> ExportSnapshot {
        let mut snapshot = ExportSnapshot::fixture();
        snapshot.processes = vec![ExportProcessMemory {
            pid: 7,
            name: "worker".into(),
            cmdline: "/bin/worker --flag=\"a,b\"\nsecond".into(),
            state: 'R',
            rss_bytes: 2048,
            ..Default::default()
        }];
        snapshot
    }

    #[test]
    fn json_round_trips_through_text() {
        let snapshot = snapshot_with_process();
        let text = render_json(&snapshot).unwrap();
        let decoded: ExportSnapshot = serde_json::from_str(&text).unwrap();
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn json_rejects_unsupported_schema_versions() {
        let mut snapshot = snapshot_with_process();
        snapshot.schema_version += 1;
        assert!(render_json(&snapshot).is_err());
    }

    #[test]
    fn csv_header_is_stable_and_comments_carry_metadata() {
        let snapshot = snapshot_with_process();
        let csv = render_csv(&snapshot, false).unwrap();
        let mut lines = csv.lines();
        let meta = lines.next().unwrap();
        assert!(meta.starts_with("# ramwise-csv schema_version=1 captured_at_unix_ms="));
        assert!(lines.any(|line| line.starts_with("# system_total_bytes=")));
        let header = csv.lines().find(|line| !line.starts_with('#')).unwrap();
        assert_eq!(header, CSV_COLUMNS.join(","));
    }

    #[test]
    fn csv_includes_all_system_fields() {
        let csv = render_csv(&snapshot_with_process(), false).unwrap();
        for field in [
            "total_bytes",
            "available_bytes",
            "free_bytes",
            "buffers_bytes",
            "cached_bytes",
            "swap_total_bytes",
            "swap_used_bytes",
            "swap_in_pages",
            "swap_out_pages",
            "swap_in_rate_per_sec",
            "swap_out_rate_per_sec",
            "slab_bytes",
            "slab_reclaimable_bytes",
            "slab_unreclaimable_bytes",
            "kernel_stack_bytes",
            "pressure_some_avg10",
            "pressure_some_avg60",
            "pressure_some_avg300",
            "pressure_full_avg10",
            "pressure_full_avg60",
            "pressure_full_avg300",
            "shared_bytes",
            "active_bytes",
            "inactive_bytes",
            "dirty_bytes",
            "writeback_bytes",
            "mapped_bytes",
        ] {
            assert!(
                csv.contains(&format!("# system_{field}=")),
                "missing {field}"
            );
        }
    }

    #[test]
    fn csv_escapes_commas_quotes_and_newlines() {
        let csv = render_csv(&snapshot_with_process(), false).unwrap();
        assert!(csv.contains("\"/bin/worker --flag=\"\"a,b\"\"\nsecond\""));
    }

    #[test]
    fn csv_escapes_carriage_returns_and_state_fields() {
        assert_eq!(escape_csv("a\rb"), "\"a\rb\"");
        let mut snapshot = snapshot_with_process();
        snapshot.processes[0].state = ',';
        let csv = render_csv(&snapshot, false).unwrap();
        assert!(csv.contains("\",\","));
    }

    #[test]
    fn details_flag_appends_regions_column_only() {
        let snapshot = snapshot_with_process();
        let plain = render_csv(&snapshot, false).unwrap();
        let detailed = render_csv(&snapshot, true).unwrap();
        let plain_header = plain.lines().find(|line| !line.starts_with('#')).unwrap();
        let detailed_header = detailed
            .lines()
            .find(|line| !line.starts_with('#'))
            .unwrap();
        assert_eq!(detailed_header, format!("{plain_header},regions_json"));
    }

    #[test]
    fn writer_refuses_to_overwrite_without_force() {
        let dir = std::env::temp_dir().join(format!(
            "ramwise-export-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos())
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("snap.json");
        std::fs::write(&path, "old").unwrap();
        assert!(write_target(&path, "new", false).is_err());
        write_target(&path, "new", true).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn writer_reports_unwritable_paths_clearly() {
        let path = Path::new("/nonexistent-ramwise-dir/sub/snap.json");
        let err = write_target(path, "data", true).unwrap_err();
        assert!(err.to_string().contains("nonexistent-ramwise-dir"));
    }
}
