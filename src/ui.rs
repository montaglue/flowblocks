use crate::{CfgLayout, EdgeKind, LayoutBlock, LayoutEdge, Point};
use egui::{
    Color32, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, pos2, vec2,
};

const MIN_VIEW_SIZE: Vec2 = vec2(360.0, 280.0);

#[derive(Clone, Debug)]
pub struct CfgViewOptions {
    pub min_size: Vec2,
    pub padding: f32,
    pub background: Color32,
    pub block_fill: Color32,
    pub block_stroke: Stroke,
    pub entry_stroke: Stroke,
    pub exit_stroke: Stroke,
    pub text_color: Color32,
    pub default_edge_stroke: Stroke,
    pub true_edge_stroke: Stroke,
    pub false_edge_stroke: Stroke,
}

impl Default for CfgViewOptions {
    fn default() -> Self {
        Self {
            min_size: MIN_VIEW_SIZE,
            padding: 28.0,
            background: Color32::from_rgb(248, 249, 251),
            block_fill: Color32::WHITE,
            block_stroke: Stroke::new(1.0, Color32::from_rgb(96, 108, 128)),
            entry_stroke: Stroke::new(2.0, Color32::from_rgb(42, 122, 89)),
            exit_stroke: Stroke::new(2.0, Color32::from_rgb(132, 82, 28)),
            text_color: Color32::from_rgb(24, 31, 42),
            default_edge_stroke: Stroke::new(1.5, Color32::from_rgb(91, 101, 116)),
            true_edge_stroke: Stroke::new(1.75, Color32::from_rgb(38, 137, 85)),
            false_edge_stroke: Stroke::new(1.75, Color32::from_rgb(181, 70, 62)),
        }
    }
}

pub fn cfg_viewer(layout: &CfgLayout) -> CfgViewer<'_> {
    CfgViewer::new(layout)
}

pub struct CfgViewer<'a> {
    layout: &'a CfgLayout,
    options: CfgViewOptions,
}

impl<'a> CfgViewer<'a> {
    pub fn new(layout: &'a CfgLayout) -> Self {
        Self {
            layout,
            options: CfgViewOptions::default(),
        }
    }

    pub fn with_options(mut self, options: CfgViewOptions) -> Self {
        self.options = options;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for CfgViewer<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let available = ui.available_size_before_wrap();
        let desired = vec2(
            available.x.max(self.options.min_size.x),
            available.y.clamp(self.options.min_size.y, 720.0),
        );
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 6.0, self.options.background);

        if self.layout.blocks.is_empty() {
            return response;
        }

        let viewport = rect.shrink(self.options.padding);
        let transform = LayoutTransform::new(self.layout, viewport);

        for edge in &self.layout.edges {
            paint_edge(&painter, edge, &transform, &self.options);
        }

        for block in &self.layout.blocks {
            paint_block(&painter, block, self.layout, &transform, &self.options);
        }

        response
    }
}

struct LayoutTransform {
    bounds: LayoutBounds,
    viewport: Rect,
    scale: f32,
}

impl LayoutTransform {
    fn new(layout: &CfgLayout, viewport: Rect) -> Self {
        let bounds = LayoutBounds::from_layout(layout);
        let bounds_size = vec2(bounds.width().max(1.0), bounds.height().max(1.0));
        let scale = (viewport.width() / bounds_size.x)
            .min(viewport.height() / bounds_size.y)
            .max(0.1);

        Self {
            bounds,
            viewport,
            scale,
        }
    }

    fn point(&self, point: Point) -> Pos2 {
        let content_size = vec2(
            self.bounds.width() * self.scale,
            self.bounds.height() * self.scale,
        );
        let origin = pos2(
            self.viewport.center().x - content_size.x / 2.0,
            self.viewport.center().y - content_size.y / 2.0,
        );

        pos2(
            origin.x + (point.x - self.bounds.min_x) * self.scale,
            origin.y + (point.y - self.bounds.min_y) * self.scale,
        )
    }

    fn size(&self, block: &LayoutBlock) -> Vec2 {
        vec2(block.size.width, block.size.height) * self.scale
    }
}

#[derive(Clone, Copy)]
struct LayoutBounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl LayoutBounds {
    fn from_layout(layout: &CfgLayout) -> Self {
        let mut bounds = Self {
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
        };

        for block in &layout.blocks {
            let size = vec2(block.size.width, block.size.height);
            bounds.include_rect(Rect::from_min_size(
                pos2(block.top_left.x, block.top_left.y),
                size,
            ));
        }

        for edge in &layout.edges {
            for point in edge.polyline() {
                bounds.include_point(pos2(point.x, point.y));
            }
        }

        if !bounds.min_x.is_finite() {
            Self {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 1.0,
                max_y: 1.0,
            }
        } else {
            bounds
        }
    }

    fn include_point(&mut self, point: Pos2) {
        self.min_x = self.min_x.min(point.x);
        self.min_y = self.min_y.min(point.y);
        self.max_x = self.max_x.max(point.x);
        self.max_y = self.max_y.max(point.y);
    }

    fn include_rect(&mut self, rect: Rect) {
        self.include_point(rect.min);
        self.include_point(rect.max);
    }

    fn width(self) -> f32 {
        self.max_x - self.min_x
    }

    fn height(self) -> f32 {
        self.max_y - self.min_y
    }
}

fn paint_edge(
    painter: &egui::Painter,
    edge: &LayoutEdge,
    transform: &LayoutTransform,
    options: &CfgViewOptions,
) {
    let stroke = match edge.kind {
        EdgeKind::Default => options.default_edge_stroke,
        EdgeKind::True => options.true_edge_stroke,
        EdgeKind::False => options.false_edge_stroke,
    };
    let points: Vec<_> = edge
        .polyline()
        .into_iter()
        .map(|point| transform.point(point))
        .collect();

    for segment in points.windows(2) {
        painter.line_segment([segment[0], segment[1]], stroke);
    }

    if let [.., previous, target] = points.as_slice() {
        paint_arrowhead(painter, *previous, *target, stroke);
    }
}

fn paint_arrowhead(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    let direction = to - from;
    let length = direction.length();
    if length <= 1.0 {
        return;
    }

    let unit = direction / length;
    let normal = vec2(-unit.y, unit.x);
    let tip = to;
    let base = to - unit * 9.0;
    painter.line_segment([tip, base + normal * 4.5], stroke);
    painter.line_segment([tip, base - normal * 4.5], stroke);
}

fn paint_block(
    painter: &egui::Painter,
    block: &LayoutBlock,
    layout: &CfgLayout,
    transform: &LayoutTransform,
    options: &CfgViewOptions,
) {
    let top_left = transform.point(block.top_left);
    let rect = Rect::from_min_size(top_left, transform.size(block));
    let stroke = if block.id == layout.entry {
        options.entry_stroke
    } else if layout.exits.contains(&block.id) {
        options.exit_stroke
    } else {
        options.block_stroke
    };

    painter.rect_filled(rect, 5.0, options.block_fill);
    painter.rect_stroke(rect, 5.0, stroke, StrokeKind::Middle);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::examples;

    #[test]
    fn viewer_can_be_constructed_for_examples() {
        for (name, cfg) in examples::all() {
            let layout = cfg
                .unwrap_or_else(|error| panic!("{name} failed to build: {error}"))
                .layout()
                .unwrap_or_else(|error| panic!("{name} failed layout: {error}"));
            let viewer = cfg_viewer(&layout);
            assert_eq!(viewer.layout.blocks.len(), layout.blocks.len());
        }
    }
}
