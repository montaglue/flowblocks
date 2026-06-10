use eframe::egui;
use flowblocks::{
    Cfg, CfgLayout, FlowblocksError, VeilMetrics, cfg_viewer, examples::all as example_cfgs,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Flowblocks CFG Viewer")
            .with_inner_size([1180.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Flowblocks CFG Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(FlowblocksViewerApp::new()))),
    )
}

struct ExampleState {
    name: &'static str,
    cfg: Cfg,
    layout: CfgLayout,
    metrics: VeilMetrics,
}

struct ExampleLoadError {
    name: &'static str,
    message: String,
}

struct FlowblocksViewerApp {
    examples: Vec<ExampleState>,
    errors: Vec<ExampleLoadError>,
    selected: usize,
}

impl FlowblocksViewerApp {
    fn new() -> Self {
        let mut examples = Vec::new();
        let mut errors = Vec::new();

        for (name, cfg) in example_cfgs() {
            match load_example(name, cfg) {
                Ok(example) => examples.push(example),
                Err(error) => errors.push(error),
            }
        }

        Self {
            examples,
            errors,
            selected: 0,
        }
    }
}

impl eframe::App for FlowblocksViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("examples")
            .resizable(false)
            .default_size(250.0)
            .show_inside(ui, |ui| {
                ui.heading("Examples");
                ui.add_space(8.0);

                for (index, example) in self.examples.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, index, example.name);
                }

                if !self.errors.is_empty() {
                    ui.separator();
                    ui.label("Load errors");
                    for error in &self.errors {
                        ui.colored_label(
                            egui::Color32::from_rgb(180, 50, 45),
                            format!("{}: {}", error.name, error.message),
                        );
                    }
                }

                ui.separator();

                if let Some(example) = self.examples.get(self.selected) {
                    ui.heading("CFG");
                    metric_row(ui, "Blocks", example.cfg.block_count());
                    metric_row(ui, "Edges", example.cfg.edge_count());
                    metric_row(ui, "Ranks", example.layout.ranks());
                    metric_row(ui, "Columns", example.layout.columns());

                    ui.separator();
                    ui.heading("VEIL");
                    metric_row(
                        ui,
                        "C1 node orth.",
                        format!("{:.3}", example.metrics.node_orthogonality),
                    );
                    metric_row(
                        ui,
                        "C2 edge orth.",
                        format!("{:.3}", example.metrics.edge_orthogonality),
                    );
                    metric_row(ui, "C3 crossings", example.metrics.edge_crossings);
                    metric_row(ui, "C4 bends", example.metrics.edge_bends);
                    metric_row(
                        ui,
                        "C5 uniformity",
                        format!("{:.3}", example.metrics.edge_uniformity_mad_log),
                    );
                    metric_row(
                        ui,
                        "C6 total len",
                        format!("{:.1}", example.metrics.short_edges.total),
                    );
                    metric_row(ui, "C7 area", format!("{:.1}", example.metrics.graph_area));
                    metric_row(
                        ui,
                        "C8 tension",
                        format!("{:.3}", example.metrics.symmetry_tension.total),
                    );
                    metric_row(
                        ui,
                        "C9 flow",
                        format!("{:.3}", example.metrics.consistent_flow),
                    );
                    metric_row(
                        ui,
                        "C10 exits",
                        format!("{:.3}", example.metrics.happens_before),
                    );
                    metric_row(
                        ui,
                        "C11 grouping",
                        format_optional(
                            example
                                .metrics
                                .edge_direction_grouping
                                .combined_median_min_distance,
                        ),
                    );
                }
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(example) = self.examples.get(self.selected) {
                ui.heading(example.name);
                ui.add_space(8.0);
                cfg_viewer(&example.layout).show(ui);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No examples loaded");
                });
            }
        });
    }
}

fn load_example(
    name: &'static str,
    cfg: Result<Cfg, FlowblocksError>,
) -> Result<ExampleState, ExampleLoadError> {
    let cfg = cfg.map_err(|error| ExampleLoadError {
        name,
        message: error.to_string(),
    })?;
    let layout = cfg.layout().map_err(|error| ExampleLoadError {
        name,
        message: error.to_string(),
    })?;
    let metrics = layout.metrics();

    Ok(ExampleState {
        name,
        cfg,
        layout,
        metrics,
    })
}

fn metric_row(ui: &mut egui::Ui, label: &str, value: impl ToString) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.monospace(value.to_string());
        });
    });
}

fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".to_owned(), |value| format!("{value:.3}"))
}
