//! Swap-activity and memory-pressure inputs.
//!
//! The live collector reads `/proc/vmstat` (`pswpin`/`pswpout` counters) and
//! `/proc/pressure/memory` (PSI averages). Both files may be missing — older
//! kernels, containers, restricted mounts — so every reader takes an explicit
//! path: the collector passes the well-known path, tests pass fixtures or a
//! missing path, and a read failure degrades to an explicit capability gap
//! instead of failing the whole snapshot.

use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result};

use super::types::MemoryPressure;

/// One `/proc/vmstat` reading of the swap counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VmstatSample {
    /// Cumulative pages swapped in.
    pub pswpin: u64,
    /// Cumulative pages swapped out.
    pub pswpout: u64,
}

/// A [`VmstatSample`] pinned to the moment it was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedSample {
    pub sample: VmstatSample,
    pub at: Instant,
}

/// Per-second swap activity derived from two consecutive samples.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwapRates {
    pub in_per_sec: f64,
    pub out_per_sec: f64,
}

/// Parse `/proc/vmstat`-formatted text. Unknown or malformed lines are
/// ignored; missing `pswpin`/`pswpout` keys read as zero.
pub fn parse_vmstat(text: &str) -> VmstatSample {
    let mut sample = VmstatSample::default();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(key), Some(raw)) = (parts.next(), parts.next()) else {
            continue;
        };
        let value = raw.parse::<i64>().unwrap_or(0).max(0) as u64;
        match key {
            "pswpin" => sample.pswpin = value,
            "pswpout" => sample.pswpout = value,
            _ => {}
        }
    }
    sample
}

/// Read and parse a vmstat file. Fails when the file is missing or
/// unreadable so the caller can record an explicit capability gap.
pub fn read_vmstat_sample(path: &Path) -> Result<VmstatSample> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(parse_vmstat(&text))
}

/// Derive per-second rates from two consecutive samples.
///
/// Returns `None` when the interval is zero or negative, or when either
/// counter moved backwards. A backwards counter means the baseline is gone
/// (reboot, kexec, namespace recycle or a wrap): the delta is unknowable,
/// so unknown is reported rather than fabricating a rate.
pub fn swap_rates(previous: &TimedSample, current: &TimedSample) -> Option<SwapRates> {
    let elapsed = current
        .at
        .saturating_duration_since(previous.at)
        .as_secs_f64();
    if elapsed <= 0.0 {
        return None;
    }
    let in_delta = current.sample.pswpin.checked_sub(previous.sample.pswpin)?;
    let out_delta = current
        .sample
        .pswpout
        .checked_sub(previous.sample.pswpout)?;
    Some(SwapRates {
        in_per_sec: in_delta as f64 / elapsed,
        out_per_sec: out_delta as f64 / elapsed,
    })
}

/// Parse `/proc/pressure/memory`-formatted text (`some`/`full` lines with
/// `avg10`/`avg60`/`avg300` tokens). Missing lines or tokens stay `None`;
/// malformed values are dropped rather than aborting the parse.
pub fn parse_memory_pressure(text: &str) -> MemoryPressure {
    let mut pressure = MemoryPressure::default();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let kind = parts.next().unwrap_or("");
        let mut avgs = [None, None, None];
        for token in parts {
            let (key, raw) = token.split_once('=').unwrap_or(("", ""));
            let slot = match key {
                "avg10" => 0,
                "avg60" => 1,
                "avg300" => 2,
                _ => continue,
            };
            // Reject non-finite values: "inf"/"nan" must not count as data.
            avgs[slot] = raw.parse::<f32>().ok().filter(|value| value.is_finite());
        }
        match kind {
            "some" => {
                pressure.some_avg10 = avgs[0];
                pressure.some_avg60 = avgs[1];
                pressure.some_avg300 = avgs[2];
            }
            "full" => {
                pressure.full_avg10 = avgs[0];
                pressure.full_avg60 = avgs[1];
                pressure.full_avg300 = avgs[2];
            }
            _ => {}
        }
    }
    pressure
}

/// Read and parse a pressure file. Fails when the file is missing or
/// unreadable so the caller can record an explicit capability gap.
pub fn read_memory_pressure(path: &Path) -> Result<MemoryPressure> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(parse_memory_pressure(&text))
}

/// Well-known live input paths.
pub const VMSTAT_PATH: &str = "/proc/vmstat";
pub const PRESSURE_MEMORY_PATH: &str = "/proc/pressure/memory";

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const VMSTAT_FIXTURE: &str = "\
nr_free_pages 123456
pswpin 1200
pswpout 3400
pgfault 999999
";

    const PRESSURE_FIXTURE: &str = "\
some avg10=1.24 avg60=0.42 avg300=0.10 total=123456
full avg10=0.31 avg60=0.12 avg300=0.02 total=23456
";

    #[test]
    fn vmstat_fixture_parses_swap_counters() {
        let sample = parse_vmstat(VMSTAT_FIXTURE);
        assert_eq!(
            sample,
            VmstatSample {
                pswpin: 1200,
                pswpout: 3400
            }
        );
    }

    #[test]
    fn vmstat_with_missing_keys_reads_as_zero() {
        assert_eq!(parse_vmstat("pgfault 42\n"), VmstatSample::default());
        assert_eq!(parse_vmstat(""), VmstatSample::default());
    }

    #[test]
    fn vmstat_ignores_malformed_lines_and_negative_values() {
        let sample = parse_vmstat("pswpin nope\npswpout -5\nbroken\npswpin 7\n");
        assert_eq!(sample.pswpin, 7);
        assert_eq!(sample.pswpout, 0);
    }

    #[test]
    fn missing_vmstat_file_is_an_error_not_a_zero_sample() {
        let missing = Path::new("/nonexistent-ramwise-fixture/vmstat");
        assert!(read_vmstat_sample(missing).is_err());
    }

    #[test]
    fn rates_come_from_sample_to_sample_deltas() {
        let start = Instant::now();
        let previous = TimedSample {
            sample: VmstatSample {
                pswpin: 1000,
                pswpout: 2000,
            },
            at: start,
        };
        let current = TimedSample {
            sample: VmstatSample {
                pswpin: 1100,
                pswpout: 2200,
            },
            at: start + Duration::from_secs(10),
        };
        let rates = swap_rates(&previous, &current).unwrap();
        assert!((rates.in_per_sec - 10.0).abs() < f64::EPSILON);
        assert!((rates.out_per_sec - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn identical_counters_mean_idle_not_unknown() {
        let at = Instant::now();
        let sample = TimedSample {
            sample: VmstatSample {
                pswpin: 50,
                pswpout: 60,
            },
            at,
        };
        let later = TimedSample {
            sample: sample.sample,
            at: at + Duration::from_secs(2),
        };
        let rates = swap_rates(&sample, &later).unwrap();
        assert_eq!(rates.in_per_sec, 0.0);
        assert_eq!(rates.out_per_sec, 0.0);
    }

    #[test]
    fn zero_or_negative_interval_has_no_rate() {
        let at = Instant::now();
        let sample = TimedSample {
            sample: VmstatSample {
                pswpin: 1,
                pswpout: 1,
            },
            at,
        };
        assert!(swap_rates(&sample, &sample).is_none());
        let earlier = TimedSample {
            sample: VmstatSample {
                pswpin: 0,
                pswpout: 0,
            },
            at,
        };
        let later = TimedSample {
            sample: VmstatSample {
                pswpin: 10,
                pswpout: 10,
            },
            at: at + Duration::from_secs(1),
        };
        assert!(swap_rates(&later, &earlier).is_none());
    }

    #[test]
    fn reset_counters_yield_unknown_not_a_rate() {
        let start = Instant::now();
        let previous = TimedSample {
            sample: VmstatSample {
                pswpin: 9000,
                pswpout: 9000,
            },
            at: start,
        };
        let current = TimedSample {
            sample: VmstatSample {
                pswpin: 100,
                pswpout: 200,
            },
            at: start + Duration::from_secs(10),
        };
        // The baseline is gone (reboot/recycle/wrap): report unknown
        // rather than fabricating a rate from the fresh counters.
        assert_eq!(swap_rates(&previous, &current), None);
    }

    #[test]
    fn pressure_fixture_parses_some_and_full_lines() {
        let pressure = parse_memory_pressure(PRESSURE_FIXTURE);
        assert!((pressure.some_avg10.unwrap() - 1.24).abs() < 0.001);
        assert!((pressure.some_avg60.unwrap() - 0.42).abs() < 0.001);
        assert!((pressure.some_avg300.unwrap() - 0.10).abs() < 0.001);
        assert!((pressure.full_avg10.unwrap() - 0.31).abs() < 0.001);
        assert!(pressure.is_available());
    }

    #[test]
    fn pressure_with_missing_full_line_keeps_some() {
        let pressure = parse_memory_pressure("some avg10=2.00 avg60=1.00 avg300=0.50 total=9\n");
        assert!(pressure.some_avg10.is_some());
        assert!(pressure.full_avg10.is_none());
        assert!(pressure.is_available());
    }

    #[test]
    fn empty_or_malformed_pressure_is_unavailable() {
        assert!(!parse_memory_pressure("").is_available());
        assert!(!parse_memory_pressure("garbage line here\n").is_available());
        let partial = parse_memory_pressure("some avg10=bogus avg60=1.00 avg300=0.50 total=9\n");
        assert!(partial.some_avg10.is_none());
        assert!(partial.some_avg60.is_some());
        assert!(partial.is_available());
    }

    #[test]
    fn non_finite_pressure_values_are_not_data() {
        let pressure = parse_memory_pressure("some avg10=inf avg60=NaN avg300=0.50 total=9\n");
        assert!(pressure.some_avg10.is_none());
        assert!(pressure.some_avg60.is_none());
        assert!(pressure.some_avg300.is_some());
        assert!(pressure.is_available());
    }

    #[test]
    fn missing_pressure_file_is_an_error_not_zero_pressure() {
        let missing = Path::new("/nonexistent-ramwise-fixture/pressure");
        assert!(read_memory_pressure(missing).is_err());
    }
}
