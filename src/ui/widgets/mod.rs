//! UI widgets

mod header;
mod process_list;
mod detail_panel;
mod graph;
mod insights_panel;
mod system_bar;

pub use header::HeaderWidget;
pub use process_list::{ProcessListWidget, ProcessListState, SortMode};
pub use detail_panel::DetailPanelWidget;
pub use graph::GraphWidget;
pub use insights_panel::InsightsPanelWidget;
