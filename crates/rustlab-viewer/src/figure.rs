//! Figure and panel state for the viewer application.

use egui_plot::{Plot, PlotBounds};
use rustlab_proto::WireSeries;

use crate::render;

/// State for a single subplot panel.
pub struct PanelState {
    pub title:  String,
    pub xlabel: String,
    pub ylabel: String,
    pub series: Vec<WireSeries>,
    pub xlim:   (Option<f64>, Option<f64>),
    pub ylim:   (Option<f64>, Option<f64>),
}

impl PanelState {
    pub fn new() -> Self {
        Self {
            title:  String::new(),
            xlabel: String::new(),
            ylabel: String::new(),
            series: Vec::new(),
            xlim:   (None, None),
            ylim:   (None, None),
        }
    }
}

/// A figure window containing a grid of subplot panels.
pub struct FigureWindow {
    pub rows:   usize,
    pub cols:   usize,
    pub title:  String,
    pub panels: Vec<PanelState>,
    /// Set to true when new data arrives; cleared after first redraw.
    pub dirty:  bool,
}

impl FigureWindow {
    pub fn new(rows: usize, cols: usize, title: String) -> Self {
        let n = rows * cols;
        let panels = (0..n).map(|_| PanelState::new()).collect();
        Self { rows, cols, title, panels, dirty: true }
    }

    /// Render this figure's subplot grid into the given `Ui`.
    pub fn render(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let cell_w = avail.x / self.cols as f32;
        let cell_h = avail.y / self.rows as f32;

        for row in 0..self.rows {
            ui.horizontal(|ui| {
                for col in 0..self.cols {
                    let idx = row * self.cols + col;
                    if idx >= self.panels.len() { continue; }
                    let panel = &self.panels[idx];

                    let title_h = if panel.title.is_empty() { 0.0 } else { 20.0 };

                    ui.vertical(|ui| {
                    if !panel.title.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(&panel.title).strong().size(14.0));
                        });
                    }

                    let plot_id = format!("fig_panel_{}_{}", row, col);
                    let mut plot = Plot::new(&plot_id)
                        .width(cell_w - 8.0)
                        .height(cell_h - 8.0 - title_h)
                        .show_axes([true, true])
                        .show_grid([true, true])
                        .allow_zoom(true)
                        .allow_drag(true)
                        .allow_scroll(true)
                        .x_axis_label(&panel.xlabel)
                        .y_axis_label(&panel.ylabel)
                        .label_formatter(|name, value| {
                            if name.is_empty() {
                                format!("x: {:.4}\ny: {:.4}", value.x, value.y)
                            } else {
                                format!("{}\nx: {:.4}\ny: {:.4}", name, value.x, value.y)
                            }
                        });

                    // Set fixed bounds when limits are specified
                    let has_bounds = panel.xlim.0.is_some() || panel.ylim.0.is_some();
                    if has_bounds {
                        // Auto-fit is disabled when explicit bounds are set
                        plot = plot.auto_bounds([
                            panel.xlim.0.is_none().into(),
                            panel.ylim.0.is_none().into(),
                        ]);
                    }

                    plot.show(ui, |plot_ui| {
                        // Apply explicit bounds
                        if let (Some(x0), Some(x1)) = panel.xlim {
                            if let (Some(y0), Some(y1)) = panel.ylim {
                                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                    [x0, y0], [x1, y1],
                                ));
                            }
                        }

                        for series in &panel.series {
                            render::render_series(plot_ui, series);
                        }
                    });
                    }); // close ui.vertical
                }
            });
        }

        self.dirty = false;
    }
}
