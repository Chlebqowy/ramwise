//! UI widgets

mod detail_panel;
mod graph;
mod header;
mod insights_panel;
mod process_list;
mod system_bar;

pub use detail_panel::DetailPanelWidget;
pub use graph::GraphWidget;
pub use header::HeaderWidget;
pub use insights_panel::InsightsPanelWidget;
pub use process_list::{ProcessListState, ProcessListWidget, SortMode};
