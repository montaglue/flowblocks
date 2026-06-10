use crate::error::{FlowblocksError, Result};
use crate::layout::{CfgLayout, LayoutBlock, LayoutEdge, Point};
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableDiGraph};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlockId(usize);

impl BlockId {
    pub const fn from_raw(raw: usize) -> Self {
        Self(raw)
    }

    pub const fn get(self) -> usize {
        self.0
    }

    fn index(self) -> NodeIndex {
        NodeIndex::new(self.0)
    }
}

impl From<NodeIndex> for BlockId {
    fn from(index: NodeIndex) -> Self {
        Self(index.index())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ControlEdgeId(usize);

impl ControlEdgeId {
    pub const fn from_raw(raw: usize) -> Self {
        Self(raw)
    }

    pub const fn get(self) -> usize {
        self.0
    }

    fn index(self) -> EdgeIndex {
        EdgeIndex::new(self.0)
    }
}

impl From<EdgeIndex> for ControlEdgeId {
    fn from(index: EdgeIndex) -> Self {
        Self(index.index())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub label: String,
    pub size: Option<BlockSize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockSize {
    pub width: f64,
    pub height: f64,
}

impl BlockSize {
    pub fn new(width: f64, height: f64) -> Result<Self> {
        if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
            Ok(Self { width, height })
        } else {
            Err(FlowblocksError::InvalidBlockSize { width, height })
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EdgeKind {
    #[default]
    Default,
    True,
    False,
}

impl EdgeKind {
    pub fn as_triskel(self) -> triskel::EdgeType {
        match self {
            Self::Default => triskel::EdgeType::Default,
            Self::True => triskel::EdgeType::True,
            Self::False => triskel::EdgeType::False,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlEdge {
    pub kind: EdgeKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CfgValidation {
    pub entry: BlockId,
    pub exits: Vec<BlockId>,
}

#[derive(Clone, Debug, Default)]
pub struct Cfg {
    graph: StableDiGraph<Block, ControlEdge>,
}

impl Cfg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_block(&mut self, label: impl Into<String>) -> BlockId {
        self.graph
            .add_node(Block {
                label: label.into(),
                size: None,
            })
            .into()
    }

    pub fn add_block_with_size(
        &mut self,
        label: impl Into<String>,
        width: f64,
        height: f64,
    ) -> Result<BlockId> {
        let size = BlockSize::new(width, height)?;
        Ok(self
            .graph
            .add_node(Block {
                label: label.into(),
                size: Some(size),
            })
            .into())
    }

    pub fn add_edge(
        &mut self,
        from: BlockId,
        to: BlockId,
        kind: EdgeKind,
    ) -> Result<ControlEdgeId> {
        self.block(from)?;
        self.block(to)?;
        Ok(self
            .graph
            .add_edge(from.index(), to.index(), ControlEdge { kind })
            .into())
    }

    pub fn block(&self, id: BlockId) -> Result<&Block> {
        self.graph
            .node_weight(id.index())
            .ok_or(FlowblocksError::InvalidBlockId(id))
    }

    pub fn edge(&self, id: ControlEdgeId) -> Result<&ControlEdge> {
        self.graph
            .edge_weight(id.index())
            .ok_or(FlowblocksError::InvalidEdgeId(id))
    }

    pub fn block_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn block_ids(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.graph.node_indices().map(BlockId::from)
    }

    pub fn edge_ids(&self) -> impl Iterator<Item = ControlEdgeId> + '_ {
        self.graph.edge_indices().map(ControlEdgeId::from)
    }

    pub fn entry(&self) -> Result<BlockId> {
        Ok(self.validate()?.entry)
    }

    pub fn exits(&self) -> Vec<BlockId> {
        self.graph
            .node_indices()
            .filter(|node| {
                self.graph
                    .neighbors_directed(*node, Outgoing)
                    .next()
                    .is_none()
            })
            .map(BlockId::from)
            .collect()
    }

    pub fn validate(&self) -> Result<CfgValidation> {
        let entries: Vec<_> = self
            .graph
            .node_indices()
            .filter(|node| {
                self.graph
                    .neighbors_directed(*node, Incoming)
                    .next()
                    .is_none()
            })
            .map(BlockId::from)
            .collect();

        let entry = match entries.as_slice() {
            [] => return Err(FlowblocksError::MissingEntry),
            [entry] => *entry,
            _ => return Err(FlowblocksError::MultipleEntries(entries)),
        };

        let exits = self.exits();
        if exits.is_empty() {
            return Err(FlowblocksError::MissingExit);
        }

        Ok(CfgValidation { entry, exits })
    }

    pub fn layout(&self) -> Result<CfgLayout> {
        let validation = self.validate()?;
        let mut builder = triskel::LayoutBuilder::new();
        let mut triskel_nodes = HashMap::with_capacity(self.graph.node_count());

        for node in self.graph.node_indices() {
            let id = BlockId::from(node);
            let block = self.block(id)?;
            let triskel_id = if let Some(size) = block.size {
                builder.make_node_with_size(size.height as f32, size.width as f32)?
            } else if block.label.is_empty() {
                builder.make_node()?
            } else {
                builder.make_node_with_label(&block.label)?
            };
            triskel_nodes.insert(id, triskel_id);
        }

        let mut triskel_edges = HashMap::with_capacity(self.graph.edge_count());
        for edge in self.graph.edge_references() {
            let id = ControlEdgeId::from(edge.id());
            let from = BlockId::from(edge.source());
            let to = BlockId::from(edge.target());
            let triskel_edge = builder.make_typed_edge(
                triskel_nodes[&from],
                triskel_nodes[&to],
                edge.weight().kind.as_triskel(),
            )?;
            triskel_edges.insert(id, triskel_edge);
        }

        let layout = builder.build()?;
        let width = finite_f32(layout.width()?)?;
        let height = finite_f32(layout.height()?)?;

        let mut blocks = Vec::with_capacity(self.graph.node_count());
        let mut coordinates = HashMap::with_capacity(self.graph.node_count());
        for node in self.graph.node_indices() {
            let id = BlockId::from(node);
            let point = Point::from_triskel(layout.coords(triskel_nodes[&id])?)?;
            coordinates.insert(id, point);
            let block = self.block(id)?.clone();
            blocks.push(LayoutBlock {
                id,
                label: block.label,
                size: block.size,
                center: point,
                rank: 0,
                column: 0,
            });
        }

        let mut edges = Vec::with_capacity(self.graph.edge_count());
        for edge in self.graph.edge_references() {
            let id = ControlEdgeId::from(edge.id());
            let from = BlockId::from(edge.source());
            let to = BlockId::from(edge.target());
            let waypoints = layout
                .waypoints(triskel_edges[&id])?
                .into_iter()
                .map(Point::from_triskel)
                .collect::<Result<Vec<_>>>()?;
            edges.push(LayoutEdge {
                id,
                from,
                to,
                kind: edge.weight().kind,
                source: coordinates[&from],
                target: coordinates[&to],
                waypoints,
            });
        }

        CfgLayout::new(
            width,
            height,
            blocks,
            edges,
            validation.entry,
            validation.exits,
        )
    }
}

fn finite_f32(value: f32) -> Result<f64> {
    let value = value as f64;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FlowblocksError::InvalidLayoutCoordinate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_single_entry_and_exit() {
        let mut cfg = Cfg::new();
        let entry = cfg.add_block("entry");
        let exit = cfg.add_block("exit");
        cfg.add_edge(entry, exit, EdgeKind::Default).unwrap();

        let validation = cfg.validate().unwrap();
        assert_eq!(validation.entry, entry);
        assert_eq!(validation.exits, vec![exit]);
    }

    #[test]
    fn rejects_multiple_entries() {
        let mut cfg = Cfg::new();
        let entry_a = cfg.add_block("entry_a");
        let entry_b = cfg.add_block("entry_b");
        let exit = cfg.add_block("exit");
        cfg.add_edge(entry_a, exit, EdgeKind::Default).unwrap();
        cfg.add_edge(entry_b, exit, EdgeKind::Default).unwrap();

        assert!(matches!(
            cfg.validate(),
            Err(FlowblocksError::MultipleEntries(_))
        ));
    }

    #[test]
    fn rejects_missing_exit() {
        let mut cfg = Cfg::new();
        let a = cfg.add_block("a");
        let b = cfg.add_block("b");
        cfg.add_edge(a, b, EdgeKind::Default).unwrap();
        cfg.add_edge(b, b, EdgeKind::Default).unwrap();

        assert!(matches!(cfg.validate(), Err(FlowblocksError::MissingExit)));
    }

    #[test]
    fn rejects_invalid_edge_endpoint() {
        let mut cfg = Cfg::new();
        let entry = cfg.add_block("entry");
        let err = cfg
            .add_edge(entry, BlockId::from_raw(999), EdgeKind::Default)
            .unwrap_err();

        assert!(matches!(err, FlowblocksError::InvalidBlockId(_)));
    }

    #[test]
    fn rejects_invalid_block_size() {
        let mut cfg = Cfg::new();
        assert!(matches!(
            cfg.add_block_with_size("bad", 0.0, 10.0),
            Err(FlowblocksError::InvalidBlockSize { .. })
        ));
    }

    #[test]
    fn lays_out_branch_cfg_with_triskel() {
        let mut cfg = Cfg::new();
        let entry = cfg.add_block("entry");
        let branch = cfg.add_block("branch");
        let true_exit = cfg.add_block("true_exit");
        let false_exit = cfg.add_block_with_size("false_exit", 20.0, 10.0).unwrap();

        cfg.add_edge(entry, branch, EdgeKind::Default).unwrap();
        cfg.add_edge(branch, true_exit, EdgeKind::True).unwrap();
        cfg.add_edge(branch, false_exit, EdgeKind::False).unwrap();

        let layout = cfg.layout().unwrap();
        assert_eq!(layout.blocks.len(), 4);
        assert_eq!(layout.edges.len(), 3);
        assert!(layout.width.is_finite());
        assert!(layout.height.is_finite());
        assert!(layout.block(entry).is_some());

        let metrics = layout.metrics();
        assert!(metrics.graph_area >= 0.0);
        assert!(metrics.edge_orthogonality.is_finite());
        assert!(metrics.consistent_flow.is_finite());
    }

    #[test]
    fn lays_out_loop_cfg_with_triskel() {
        let mut cfg = Cfg::new();
        let entry = cfg.add_block("entry");
        let header = cfg.add_block("header");
        let body = cfg.add_block("body");
        let exit = cfg.add_block("exit");

        cfg.add_edge(entry, header, EdgeKind::Default).unwrap();
        cfg.add_edge(header, body, EdgeKind::True).unwrap();
        cfg.add_edge(body, header, EdgeKind::Default).unwrap();
        cfg.add_edge(header, exit, EdgeKind::False).unwrap();

        let layout = cfg.layout().unwrap();
        assert_eq!(layout.blocks.len(), 4);
        assert_eq!(layout.edges.len(), 4);
        assert!(layout.edges.iter().all(|edge| edge.length().is_finite()));

        let metrics = layout.metrics();
        assert!(metrics.consistent_flow < 1.0);
        assert!(metrics.happens_before.is_finite());
    }
}
