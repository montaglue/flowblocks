use crate::cfg::BlockId;
use crate::layout::{CfgLayout, LayoutEdge, Point};
use std::cmp::Ordering;
use std::collections::HashMap;

const EPSILON: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeLengthSummary {
    pub total: f64,
    pub max: f64,
    pub median: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SymmetryTension {
    pub total: f64,
    pub median: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DirectionGrouping {
    pub forward_median_min_distance: Option<f64>,
    pub back_median_min_distance: Option<f64>,
    pub combined_median_min_distance: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VeilMetrics {
    pub node_orthogonality: f64,
    pub edge_orthogonality: f64,
    pub edge_crossings: usize,
    pub edge_bends: usize,
    pub edge_uniformity_mad_log: f64,
    pub short_edges: EdgeLengthSummary,
    pub graph_area: f64,
    pub symmetry_tension: SymmetryTension,
    pub consistent_flow: f64,
    pub happens_before: f64,
    pub edge_direction_grouping: DirectionGrouping,
}

impl VeilMetrics {
    pub fn compute(layout: &CfgLayout) -> Self {
        let block_map = layout.block_map();
        let lengths = routed_edge_lengths(layout);

        Self {
            node_orthogonality: node_orthogonality(layout),
            edge_orthogonality: edge_orthogonality(layout),
            edge_crossings: edge_crossings(layout),
            edge_bends: edge_bends(layout),
            edge_uniformity_mad_log: edge_uniformity_mad_log(&lengths),
            short_edges: short_edges(&lengths),
            graph_area: graph_area(layout),
            symmetry_tension: symmetry_tension(layout, &block_map),
            consistent_flow: consistent_flow(layout, &block_map),
            happens_before: happens_before(layout, &block_map),
            edge_direction_grouping: edge_direction_grouping(layout, &block_map),
        }
    }
}

fn node_orthogonality(layout: &CfgLayout) -> f64 {
    let area = layout.ranks() * layout.columns();
    if area == 0 {
        0.0
    } else {
        layout.blocks.len() as f64 / area as f64
    }
}

fn edge_orthogonality(layout: &CfgLayout) -> f64 {
    let mut scores = Vec::new();
    for edge in &layout.edges {
        for segment in edge.polyline().windows(2) {
            let dx = (segment[1].x - segment[0].x).abs();
            let dy = (segment[1].y - segment[0].y).abs();
            let length = dx.hypot(dy);
            if length > EPSILON {
                scores.push(dx.max(dy) / length);
            }
        }
    }

    mean(&scores)
}

fn edge_crossings(layout: &CfgLayout) -> usize {
    let segments: Vec<_> = layout
        .edges
        .iter()
        .flat_map(|edge| {
            edge.polyline()
                .windows(2)
                .map(|pair| (edge.id.get(), pair[0], pair[1]))
                .collect::<Vec<_>>()
        })
        .collect();

    let mut crossings = 0;
    for left_index in 0..segments.len() {
        for right_index in (left_index + 1)..segments.len() {
            let (left_edge, a, b) = segments[left_index];
            let (right_edge, c, d) = segments[right_index];
            if left_edge == right_edge || shares_endpoint(a, b, c, d) {
                continue;
            }
            if segments_intersect(a, b, c, d) {
                crossings += 1;
            }
        }
    }

    crossings
}

fn edge_bends(layout: &CfgLayout) -> usize {
    layout
        .edges
        .iter()
        .map(|edge| {
            edge.polyline()
                .windows(3)
                .filter(|triple| !collinear(triple[0], triple[1], triple[2]))
                .count()
        })
        .sum()
}

fn routed_edge_lengths(layout: &CfgLayout) -> Vec<f64> {
    layout.edges.iter().map(LayoutEdge::length).collect()
}

fn edge_uniformity_mad_log(lengths: &[f64]) -> f64 {
    let logs: Vec<_> = lengths
        .iter()
        .copied()
        .filter(|length| *length > EPSILON)
        .map(f64::ln)
        .collect();
    if logs.is_empty() {
        return 0.0;
    }

    let median_log = median(logs.clone());
    let deviations: Vec<_> = logs
        .into_iter()
        .map(|value| (value - median_log).abs())
        .collect();
    median(deviations)
}

fn short_edges(lengths: &[f64]) -> EdgeLengthSummary {
    EdgeLengthSummary {
        total: lengths.iter().sum(),
        max: lengths.iter().copied().fold(0.0, f64::max),
        median: median(lengths.to_vec()),
    }
}

fn graph_area(layout: &CfgLayout) -> f64 {
    let area = layout.width * layout.height;
    if area.is_finite() && area > 0.0 {
        area
    } else {
        bounding_box_area(layout)
    }
}

fn symmetry_tension(
    layout: &CfgLayout,
    block_map: &HashMap<BlockId, &crate::layout::LayoutBlock>,
) -> SymmetryTension {
    let ideal = ideal_edge_length(layout, block_map);
    if ideal <= EPSILON {
        return SymmetryTension::default();
    }

    let mut tension_by_block = HashMap::<BlockId, f64>::new();
    for edge in &layout.edges {
        let distance = edge.source.distance(edge.target);
        let tension = (distance - ideal).abs() / ideal;
        *tension_by_block.entry(edge.from).or_default() += tension;
        *tension_by_block.entry(edge.to).or_default() += tension;
    }

    let values: Vec<_> = layout
        .blocks
        .iter()
        .map(|block| tension_by_block.get(&block.id).copied().unwrap_or_default())
        .collect();

    SymmetryTension {
        total: values.iter().sum(),
        median: median(values),
    }
}

fn consistent_flow(
    layout: &CfgLayout,
    block_map: &HashMap<BlockId, &crate::layout::LayoutBlock>,
) -> f64 {
    if layout.edges.is_empty() {
        return 1.0;
    }

    let consistent = layout
        .edges
        .iter()
        .filter(|edge| block_map[&edge.from].rank < block_map[&edge.to].rank)
        .count();
    consistent as f64 / layout.edges.len() as f64
}

fn happens_before(
    layout: &CfgLayout,
    block_map: &HashMap<BlockId, &crate::layout::LayoutBlock>,
) -> f64 {
    if layout.blocks.is_empty() {
        return 0.0;
    }

    let max_rank = layout.ranks().saturating_sub(1);
    if max_rank == 0 {
        return 1.0;
    }

    layout
        .exits
        .iter()
        .filter_map(|exit| {
            block_map
                .get(exit)
                .map(|block| block.rank as f64 / max_rank as f64)
        })
        .fold(1.0, f64::min)
}

fn edge_direction_grouping(
    layout: &CfgLayout,
    block_map: &HashMap<BlockId, &crate::layout::LayoutBlock>,
) -> DirectionGrouping {
    let classified = layout
        .edges
        .iter()
        .filter_map(|edge| {
            let from = block_map[&edge.from];
            let to = block_map[&edge.to];
            if from.rank == to.rank {
                return None;
            }

            Some(ClassifiedEdge {
                forward: from.rank < to.rank,
                min_rank: from.rank.min(to.rank),
                max_rank: from.rank.max(to.rank),
                lane_x: (edge.source.x + edge.target.x) / 2.0,
            })
        })
        .collect::<Vec<_>>();

    let forward = median_min_lane_distance(classified.iter().copied().filter(|edge| edge.forward));
    let back = median_min_lane_distance(classified.iter().copied().filter(|edge| !edge.forward));
    let combined = median_min_lane_distance(classified.iter().copied());

    DirectionGrouping {
        forward_median_min_distance: forward,
        back_median_min_distance: back,
        combined_median_min_distance: combined,
    }
}

#[derive(Clone, Copy)]
struct ClassifiedEdge {
    forward: bool,
    min_rank: usize,
    max_rank: usize,
    lane_x: f64,
}

fn median_min_lane_distance(edges: impl Iterator<Item = ClassifiedEdge>) -> Option<f64> {
    let edges: Vec<_> = edges.collect();
    let mut distances = Vec::new();

    for (index, edge) in edges.iter().enumerate() {
        let mut best = None::<f64>;
        for (other_index, other) in edges.iter().enumerate() {
            if index == other_index || !rank_ranges_overlap(*edge, *other) {
                continue;
            }
            let distance = (edge.lane_x - other.lane_x).abs();
            best = Some(best.map_or(distance, |current| current.min(distance)));
        }
        if let Some(best) = best {
            distances.push(best);
        }
    }

    (!distances.is_empty()).then(|| median(distances))
}

fn rank_ranges_overlap(left: ClassifiedEdge, right: ClassifiedEdge) -> bool {
    left.min_rank <= right.max_rank && right.min_rank <= left.max_rank
}

fn ideal_edge_length(
    layout: &CfgLayout,
    block_map: &HashMap<BlockId, &crate::layout::LayoutBlock>,
) -> f64 {
    let mut rank_y = HashMap::<usize, Vec<f64>>::new();
    for block in &layout.blocks {
        rank_y.entry(block.rank).or_default().push(block.center.y);
    }

    let mut rank_centers: Vec<_> = rank_y
        .into_iter()
        .map(|(rank, values)| (rank, mean(&values)))
        .collect();
    rank_centers.sort_by_key(|(rank, _)| *rank);

    let spacings: Vec<_> = rank_centers
        .windows(2)
        .map(|pair| (pair[1].1 - pair[0].1).abs())
        .filter(|spacing| *spacing > EPSILON)
        .collect();
    if !spacings.is_empty() {
        return median(spacings);
    }

    let center_lengths: Vec<_> = layout
        .edges
        .iter()
        .map(|edge| {
            block_map[&edge.from]
                .center
                .distance(block_map[&edge.to].center)
        })
        .filter(|length| *length > EPSILON)
        .collect();

    if center_lengths.is_empty() {
        1.0
    } else {
        median(center_lengths)
    }
}

fn bounding_box_area(layout: &CfgLayout) -> f64 {
    let points: Vec<_> = layout
        .blocks
        .iter()
        .map(|block| block.center)
        .chain(layout.edges.iter().flat_map(|edge| edge.polyline()))
        .collect();

    if points.is_empty() {
        return 0.0;
    }

    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max);

    (max_x - min_x).max(0.0) * (max_y - min_y).max(0.0)
}

fn shares_endpoint(a: Point, b: Point, c: Point, d: Point) -> bool {
    same_point(a, c) || same_point(a, d) || same_point(b, c) || same_point(b, d)
}

fn same_point(a: Point, b: Point) -> bool {
    (a.x - b.x).abs() <= EPSILON && (a.y - b.y).abs() <= EPSILON
}

fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> bool {
    let o1 = orientation(a, b, c);
    let o2 = orientation(a, b, d);
    let o3 = orientation(c, d, a);
    let o4 = orientation(c, d, b);

    if o1 != o2 && o3 != o4 {
        return true;
    }

    (o1 == 0 && on_segment(a, c, b))
        || (o2 == 0 && on_segment(a, d, b))
        || (o3 == 0 && on_segment(c, a, d))
        || (o4 == 0 && on_segment(c, b, d))
}

fn orientation(a: Point, b: Point, c: Point) -> i8 {
    let value = (b.y - a.y) * (c.x - b.x) - (b.x - a.x) * (c.y - b.y);
    if value.abs() <= EPSILON {
        0
    } else if value > 0.0 {
        1
    } else {
        -1
    }
}

fn on_segment(a: Point, b: Point, c: Point) -> bool {
    b.x >= a.x.min(c.x) - EPSILON
        && b.x <= a.x.max(c.x) + EPSILON
        && b.y >= a.y.min(c.y) - EPSILON
        && b.y <= a.y.max(c.y) + EPSILON
}

fn collinear(a: Point, b: Point, c: Point) -> bool {
    ((b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)).abs() <= EPSILON
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn median(mut values: Vec<f64>) -> f64 {
    values.retain(|value| value.is_finite());
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::{BlockSize, ControlEdgeId, EdgeKind};
    use crate::layout::{LayoutBlock, LayoutEdge};

    #[test]
    fn computes_deterministic_metrics_for_fixture() {
        let layout = fixture_layout();
        let metrics = VeilMetrics::compute(&layout);

        assert_eq!(metrics.node_orthogonality, 2.0 / 3.0);
        assert_eq!(metrics.edge_crossings, 1);
        assert_eq!(metrics.edge_bends, 2);
        assert_eq!(metrics.graph_area, 10_000.0);
        assert_eq!(metrics.consistent_flow, 2.0 / 3.0);
        assert_eq!(metrics.happens_before, 1.0);
        assert!(metrics.edge_orthogonality > 0.7);
        assert!(metrics.short_edges.total > 0.0);
        assert!(metrics.edge_uniformity_mad_log >= 0.0);
        assert!(metrics.symmetry_tension.total >= 0.0);
    }

    #[test]
    fn detects_back_edge_flow() {
        let mut layout = fixture_layout();
        layout.edges.push(LayoutEdge {
            id: ControlEdgeId::from_raw(99),
            from: BlockId::from_raw(3),
            to: BlockId::from_raw(0),
            kind: EdgeKind::Default,
            source: Point { x: 100.0, y: 100.0 },
            target: Point { x: 0.0, y: 0.0 },
            waypoints: vec![],
        });

        let metrics = VeilMetrics::compute(&layout);
        assert!(metrics.consistent_flow < 1.0);
        assert!(
            metrics
                .edge_direction_grouping
                .combined_median_min_distance
                .is_some()
        );
    }

    #[test]
    fn empty_layout_metrics_are_defined() {
        let layout =
            CfgLayout::new(0.0, 0.0, vec![], vec![], BlockId::from_raw(0), vec![]).unwrap();
        let metrics = VeilMetrics::compute(&layout);

        assert_eq!(metrics.node_orthogonality, 0.0);
        assert_eq!(metrics.consistent_flow, 1.0);
        assert_eq!(metrics.happens_before, 0.0);
    }

    fn fixture_layout() -> CfgLayout {
        CfgLayout::new(
            100.0,
            100.0,
            vec![
                block(0, 0.0, 0.0),
                block(1, 0.0, 50.0),
                block(2, 100.0, 50.0),
                block(3, 100.0, 100.0),
            ],
            vec![
                edge(0, 0, 3, (0.0, 0.0), (100.0, 100.0), vec![]),
                edge(1, 2, 1, (100.0, 50.0), (0.0, 50.0), vec![(50.0, 75.0)]),
                edge(2, 0, 2, (0.0, 0.0), (100.0, 50.0), vec![(50.0, 0.0)]),
            ],
            BlockId::from_raw(0),
            vec![BlockId::from_raw(3)],
        )
        .unwrap()
    }

    fn block(id: usize, x: f64, y: f64) -> LayoutBlock {
        LayoutBlock {
            id: BlockId::from_raw(id),
            label: id.to_string(),
            size: Some(BlockSize {
                width: 10.0,
                height: 10.0,
            }),
            center: Point { x, y },
            rank: 0,
            column: 0,
        }
    }

    fn edge(
        id: usize,
        from: usize,
        to: usize,
        source: (f64, f64),
        target: (f64, f64),
        waypoints: Vec<(f64, f64)>,
    ) -> LayoutEdge {
        LayoutEdge {
            id: ControlEdgeId::from_raw(id),
            from: BlockId::from_raw(from),
            to: BlockId::from_raw(to),
            kind: EdgeKind::Default,
            source: Point {
                x: source.0,
                y: source.1,
            },
            target: Point {
                x: target.0,
                y: target.1,
            },
            waypoints: waypoints.into_iter().map(|(x, y)| Point { x, y }).collect(),
        }
    }

    #[test]
    fn median_ignores_non_finite_values() {
        let values = vec![1.0, f64::NAN, 3.0];
        assert_eq!(median(values), 2.0);
    }

    #[test]
    fn polyline_length_sums_segments() {
        let points = [
            Point { x: 0.0, y: 0.0 },
            Point { x: 3.0, y: 4.0 },
            Point { x: 6.0, y: 8.0 },
        ];
        assert_eq!(crate::layout::polyline_length(&points), 10.0);
    }

    #[test]
    fn crossing_excludes_shared_endpoint() {
        let a = Point { x: 0.0, y: 0.0 };
        let b = Point { x: 1.0, y: 1.0 };
        let c = Point { x: 1.0, y: 1.0 };
        let d = Point { x: 2.0, y: 0.0 };

        assert!(segments_intersect(a, b, c, d));
        assert!(shares_endpoint(a, b, c, d));
    }
}
