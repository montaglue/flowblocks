//! Control-flow graph layout and VEIL metrics.

pub mod cfg;
pub mod error;
pub mod examples;
pub mod layout;
pub mod metrics;
#[cfg(feature = "ui")]
pub mod ui;

pub use cfg::{
    Block, BlockId, BlockSize, Cfg, CfgValidation, ControlEdge, ControlEdgeId, EdgeKind,
};
pub use error::{FlowblocksError, Result};
pub use layout::{CfgLayout, LayoutBlock, LayoutEdge, Point};
pub use metrics::{DirectionGrouping, EdgeLengthSummary, SymmetryTension, VeilMetrics};
#[cfg(feature = "ui")]
pub use ui::{CfgViewOptions, CfgViewer, cfg_viewer};
