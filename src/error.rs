use crate::cfg::{BlockId, ControlEdgeId};
use std::fmt;

pub type Result<T> = std::result::Result<T, FlowblocksError>;

#[derive(Debug)]
pub enum FlowblocksError {
    InvalidBlockId(BlockId),
    InvalidEdgeId(ControlEdgeId),
    InvalidBlockSize { width: f64, height: f64 },
    MissingEntry,
    MultipleEntries(Vec<BlockId>),
    MissingExit,
    InvalidLayoutCoordinate,
    Triskel(triskel::Error),
}

impl fmt::Display for FlowblocksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBlockId(id) => write!(f, "invalid block id {}", id.get()),
            Self::InvalidEdgeId(id) => write!(f, "invalid edge id {}", id.get()),
            Self::InvalidBlockSize { width, height } => {
                write!(f, "invalid block size {width}x{height}")
            }
            Self::MissingEntry => f.write_str("CFG must have exactly one entry block, found none"),
            Self::MultipleEntries(entries) => {
                write!(f, "CFG must have exactly one entry block, found ")?;
                for (index, entry) in entries.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}", entry.get())?;
                }
                Ok(())
            }
            Self::MissingExit => f.write_str("CFG must have at least one exit block"),
            Self::InvalidLayoutCoordinate => f.write_str("layout contains a non-finite coordinate"),
            Self::Triskel(error) => write!(f, "triskel layout failed: {error}"),
        }
    }
}

impl std::error::Error for FlowblocksError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Triskel(error) => Some(error),
            _ => None,
        }
    }
}

impl From<triskel::Error> for FlowblocksError {
    fn from(error: triskel::Error) -> Self {
        Self::Triskel(error)
    }
}
