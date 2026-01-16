//! History buffer for time-series memory data
//!
//! Maintains a rolling window of memory snapshots for trend analysis
//! and graph visualization.

mod buffer;

pub use buffer::HistoryBuffer;
