//! Main eframe application for rustlab-viewer.

use std::collections::HashMap;
use std::sync::mpsc;
use rustlab_proto::ViewerMsg;

use crate::figure::FigureWindow;

/// The viewer application state.
pub struct ViewerApp {
    rx:      mpsc::Receiver<ViewerMsg>,
    figures: HashMap<u32, FigureWindow>,
}

impl ViewerApp {
    pub fn new(rx: mpsc::Receiver<ViewerMsg>) -> Self {
        Self {
            rx,
            figures: HashMap::new(),
        }
    }

    /// Drain all pending messages from the socket listener.
    fn process_messages(&mut self, ctx: &egui::Context) {
        let mut any_update = false;
        while let Ok(msg) = self.rx.try_recv() {
            any_update = true;
            match msg {
                ViewerMsg::FigureOpen { id, rows, cols, title } => {
                    self.figures.insert(
                        id,
                        FigureWindow::new(rows as usize, cols as usize, title),
                    );
                }
                ViewerMsg::PanelUpdate { fig_id, panel, series } => {
                    if let Some(fig) = self.figures.get_mut(&fig_id) {
                        let idx = panel as usize;
                        if idx < fig.panels.len() {
                            fig.panels[idx].series = series;
                            fig.dirty = true;
                        }
                    }
                }
                ViewerMsg::PanelLabels { fig_id, panel, title, xlabel, ylabel } => {
                    if let Some(fig) = self.figures.get_mut(&fig_id) {
                        let idx = panel as usize;
                        if idx < fig.panels.len() {
                            fig.panels[idx].title = title;
                            fig.panels[idx].xlabel = xlabel;
                            fig.panels[idx].ylabel = ylabel;
                        }
                    }
                }
                ViewerMsg::PanelLimits { fig_id, panel, xlim, ylim } => {
                    if let Some(fig) = self.figures.get_mut(&fig_id) {
                        let idx = panel as usize;
                        if idx < fig.panels.len() {
                            fig.panels[idx].xlim = xlim;
                            fig.panels[idx].ylim = ylim;
                        }
                    }
                }
                ViewerMsg::Redraw { fig_id } => {
                    if let Some(fig) = self.figures.get_mut(&fig_id) {
                        fig.dirty = true;
                    }
                }
                ViewerMsg::Close { fig_id } => {
                    self.figures.remove(&fig_id);
                }
                ViewerMsg::Ping => {} // handled at the connection level
            }
        }
        if any_update {
            ctx.request_repaint();
        }
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages(ctx);

        // Request periodic repaint so we pick up new messages promptly.
        // When idle this costs almost nothing on modern GPUs.
        ctx.request_repaint_after(std::time::Duration::from_millis(16));

        // Dark theme
        ctx.set_visuals(egui::Visuals::dark());

        if self.figures.is_empty() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label("Waiting for rustlab connection...");
                });
            });
            return;
        }

        // Render each figure in an egui Window (or central panel if only one)
        if self.figures.len() == 1 {
            let (&id, fig) = self.figures.iter_mut().next().unwrap();
            egui::CentralPanel::default().show(ctx, |ui| {
                if !fig.title.is_empty() {
                    ui.heading(&fig.title);
                }
                fig.render(ui, id);
            });
        } else {
            egui::CentralPanel::default().show(ctx, |_ui| {});
            let ids: Vec<u32> = self.figures.keys().copied().collect();
            for id in ids {
                let fig = self.figures.get_mut(&id).unwrap();
                let title = if fig.title.is_empty() {
                    format!("Figure {}", id)
                } else {
                    fig.title.clone()
                };
                egui::Window::new(&title)
                    .id(egui::Id::new(format!("fig_{}", id)))
                    .resizable(true)
                    .show(ctx, |ui| {
                        fig.render(ui, id);
                    });
            }
        }
    }
}
