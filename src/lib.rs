//! Control-flow graph layout and VEIL metrics.

pub mod cfg;
pub mod error;
pub mod layout;
pub mod metrics;

pub use cfg::{
    Block, BlockId, BlockSize, Cfg, CfgValidation, ControlEdge, ControlEdgeId, EdgeKind,
};
pub use error::{FlowblocksError, Result};
pub use layout::{CfgLayout, LayoutBlock, LayoutEdge, Point};
pub use metrics::{DirectionGrouping, EdgeLengthSummary, SymmetryTension, VeilMetrics};
