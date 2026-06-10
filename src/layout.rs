use crate::cfg::{BlockId, BlockSize, ControlEdgeId, EdgeKind};
use crate::error::{FlowblocksError, Result};
use crate::metrics::VeilMetrics;
use std::cmp::Ordering;
use std::collections::HashMap;

const CLUSTER_EPSILON: f64 = 1.0e-3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Result<Self> {
        if x.is_finite() && y.is_finite() {
            Ok(Self { x, y })
        } else {
            Err(FlowblocksError::InvalidLayoutCoordinate)
        }
    }

    pub(crate) fn from_triskel(point: triskel::Point) -> Result<Self> {
        Self::new(point.x as f64, point.y as f64)
    }

    pub(crate) fn distance(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.hypot(dy)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBlock {
    pub id: BlockId,
    pub label: String,
    pub size: Option<BlockSize>,
    pub top_left: Point,
    pub rank: usize,
    pub column: usize,
}

impl LayoutBlock {
    pub fn center(&self) -> Point {
        let size = self.size.unwrap_or(BlockSize {
            width: 1.0,
            height: 1.0,
        });
        Point {
            x: self.top_left.x + size.width / 2.0,
            y: self.top_left.y + size.height / 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutEdge {
    pub id: ControlEdgeId,
    pub from: BlockId,
    pub to: BlockId,
    pub kind: EdgeKind,
    pub source: Point,
    pub target: Point,
    pub waypoints: Vec<Point>,
}

impl LayoutEdge {
    pub fn polyline(&self) -> Vec<Point> {
        let mut points = Vec::with_capacity(self.waypoints.len().max(2));
        for point in &self.waypoints {
            push_distinct(&mut points, *point);
        }
        if points.is_empty() {
            push_distinct(&mut points, self.source);
            push_distinct(&mut points, self.target);
        }
        points
    }

    pub fn length(&self) -> f64 {
        polyline_length(&self.polyline())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CfgLayout {
    pub width: f64,
    pub height: f64,
    pub blocks: Vec<LayoutBlock>,
    pub edges: Vec<LayoutEdge>,
    pub entry: BlockId,
    pub exits: Vec<BlockId>,
}

impl CfgLayout {
    pub fn new(
        width: f64,
        height: f64,
        mut blocks: Vec<LayoutBlock>,
        edges: Vec<LayoutEdge>,
        entry: BlockId,
        exits: Vec<BlockId>,
    ) -> Result<Self> {
        if !width.is_finite() || !height.is_finite() {
            return Err(FlowblocksError::InvalidLayoutCoordinate);
        }

        infer_grid_positions(&mut blocks);

        Ok(Self {
            width,
            height,
            blocks,
            edges,
            entry,
            exits,
        })
    }

    pub fn block(&self, id: BlockId) -> Option<&LayoutBlock> {
        self.blocks.iter().find(|block| block.id == id)
    }

    pub fn edge(&self, id: ControlEdgeId) -> Option<&LayoutEdge> {
        self.edges.iter().find(|edge| edge.id == id)
    }

    pub fn ranks(&self) -> usize {
        self.blocks
            .iter()
            .map(|block| block.rank)
            .max()
            .map_or(0, |rank| rank + 1)
    }

    pub fn columns(&self) -> usize {
        self.blocks
            .iter()
            .map(|block| block.column)
            .max()
            .map_or(0, |column| column + 1)
    }

    pub fn block_map(&self) -> HashMap<BlockId, &LayoutBlock> {
        self.blocks.iter().map(|block| (block.id, block)).collect()
    }

    pub fn metrics(&self) -> VeilMetrics {
        VeilMetrics::compute(self)
    }
}

pub(crate) fn polyline_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum()
}

fn push_distinct(points: &mut Vec<Point>, point: Point) {
    if points
        .last()
        .is_none_or(|last| last.distance(point) > CLUSTER_EPSILON)
    {
        points.push(point);
    }
}

fn infer_grid_positions(blocks: &mut [LayoutBlock]) {
    let ys = sorted_axis_values(blocks.iter().map(|block| block.top_left.y));
    let xs = sorted_axis_values(blocks.iter().map(|block| block.top_left.x));

    for block in blocks {
        block.rank = nearest_cluster(block.top_left.y, &ys);
        block.column = nearest_cluster(block.top_left.x, &xs);
    }
}

fn sorted_axis_values(values: impl Iterator<Item = f64>) -> Vec<f64> {
    let mut clusters = Vec::new();
    let mut values: Vec<_> = values.filter(|value| value.is_finite()).collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    for value in values {
        if clusters
            .last()
            .is_none_or(|last: &f64| (value - *last).abs() > CLUSTER_EPSILON)
        {
            clusters.push(value);
        }
    }

    clusters
}

fn nearest_cluster(value: f64, clusters: &[f64]) -> usize {
    clusters
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (value - **a)
                .abs()
                .partial_cmp(&(value - **b).abs())
                .unwrap_or(Ordering::Equal)
        })
        .map_or(0, |(index, _)| index)
}
