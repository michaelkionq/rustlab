//! `ViewerFigure` — a `LivePlot` backend that sends data to `rustlab-viewer`
//! over a Unix socket instead of rendering in the terminal.
//!
//! Also provides `connect_viewer()` / `disconnect_viewer()` / `viewer_active()`
//! / `sync_viewer()` for routing regular (non-live) plot commands to the viewer.

use crate::viewer_client::ViewerClient;
use crate::figure::{FigureState, LineStyle, PlotKind, SeriesColor, FIGURE};
use crate::{LivePlot, PlotError};
use rustlab_proto::{ViewerMsg, WireColor, WireLineStyle, WirePlotKind, WireSeries};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_FIG_ID: AtomicU32 = AtomicU32::new(0);

/// Initialize figure ID counter with a PID-based prefix to avoid collisions
/// when multiple rustlab processes connect to the same viewer.
/// Layout: upper 16 bits = PID (truncated), lower 16 bits = local counter.
fn next_fig_id() -> u32 {
    let local = NEXT_FIG_ID.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id() as u32;
    (pid << 16) | (local & 0xFFFF)
}

/// A live figure backed by the external `rustlab-viewer` process.
#[derive(Debug)]
pub struct ViewerFigure {
    client: ViewerClient,
    fig_id: u32,
}

impl ViewerFigure {
    /// Try to connect to a running viewer and open a figure.
    /// Returns `None` if the viewer is not running.
    pub fn connect(rows: usize, cols: usize) -> Option<Self> {
        let mut client = ViewerClient::connect()?;
        let fig_id = next_fig_id();
        let msg = ViewerMsg::FigureOpen {
            id:    fig_id,
            rows:  rows as u16,
            cols:  cols as u16,
            title: String::new(),
        };
        client.send(&msg).ok()?;
        Some(Self { client, fig_id })
    }
}

impl LivePlot for ViewerFigure {
    fn update_panel(&mut self, idx: usize, x: Vec<f64>, y: Vec<f64>) {
        let msg = ViewerMsg::PanelUpdate {
            fig_id: self.fig_id,
            panel:  idx as u16,
            series: vec![WireSeries {
                label: String::new(),
                x,
                y,
                color: WireColor::Named("cyan".into()),
                style: WireLineStyle::Solid,
                kind:  WirePlotKind::Line,
                x_labels: None,
            }],
        };
        let _ = self.client.send_nowait(&msg);
    }

    fn set_panel_labels(&mut self, idx: usize, title: &str, xlabel: &str, ylabel: &str) {
        let msg = ViewerMsg::PanelLabels {
            fig_id: self.fig_id,
            panel:  idx as u16,
            title:  title.to_string(),
            xlabel: xlabel.to_string(),
            ylabel: ylabel.to_string(),
        };
        let _ = self.client.send_nowait(&msg);
    }

    fn set_panel_limits(&mut self, idx: usize, xlim: (Option<f64>, Option<f64>), ylim: (Option<f64>, Option<f64>)) {
        let msg = ViewerMsg::PanelLimits {
            fig_id: self.fig_id,
            panel:  idx as u16,
            xlim,
            ylim,
        };
        let _ = self.client.send_nowait(&msg);
    }

    fn redraw(&mut self) -> Result<(), PlotError> {
        let msg = ViewerMsg::Redraw { fig_id: self.fig_id };
        self.client.send(&msg)?;
        Ok(())
    }
}

impl Drop for ViewerFigure {
    fn drop(&mut self) {
        let msg = ViewerMsg::Close { fig_id: self.fig_id };
        let _ = self.client.send_nowait(&msg);
    }
}

// ─── Conversion helpers ─────────────────────────────────────────────────────

pub fn color_to_wire(c: &SeriesColor) -> WireColor {
    match c {
        SeriesColor::Blue    => WireColor::Named("blue".into()),
        SeriesColor::Red     => WireColor::Named("red".into()),
        SeriesColor::Green   => WireColor::Named("green".into()),
        SeriesColor::Cyan    => WireColor::Named("cyan".into()),
        SeriesColor::Magenta => WireColor::Named("magenta".into()),
        SeriesColor::Yellow  => WireColor::Named("yellow".into()),
        SeriesColor::Black   => WireColor::Named("black".into()),
        SeriesColor::White   => WireColor::Named("white".into()),
        SeriesColor::Rgb(r, g, b) => WireColor::Rgb(*r, *g, *b),
    }
}

pub fn style_to_wire(s: &LineStyle) -> WireLineStyle {
    match s {
        LineStyle::Solid  => WireLineStyle::Solid,
        LineStyle::Dashed => WireLineStyle::Dashed,
    }
}

pub fn kind_to_wire(k: &PlotKind) -> WirePlotKind {
    match k {
        PlotKind::Line    => WirePlotKind::Line,
        PlotKind::Stem    => WirePlotKind::Stem,
        PlotKind::Bar     => WirePlotKind::Bar,
        PlotKind::Scatter => WirePlotKind::Scatter,
    }
}

// ─── Viewer connection for regular (non-live) plotting ─────────────────────

struct ViewerConn {
    client: ViewerClient,
    fig_id: u32,
    /// Last-sent subplot layout (rows, cols) — avoid resending FigureOpen.
    layout: (usize, usize),
}

thread_local! {
    /// Active viewer connection for regular plot commands.
    static VIEWER_CONN: RefCell<Option<ViewerConn>> = RefCell::new(None);
}

/// Try to connect to a running viewer. Returns Ok(true) if connected,
/// Ok(false) if the viewer is not running.
pub fn connect_viewer() -> Result<bool, PlotError> {
    connect_viewer_impl(ViewerClient::connect())
}

/// Connect to a named viewer session (e.g. `viewer on work`).
pub fn connect_viewer_named(name: &str) -> Result<bool, PlotError> {
    connect_viewer_impl(ViewerClient::connect_named(name))
}

fn connect_viewer_impl(client: Option<ViewerClient>) -> Result<bool, PlotError> {
    let Some(mut client) = client else {
        return Ok(false);
    };
    // Verify the connection is live
    match client.send(&ViewerMsg::Ping) {
        Ok(rustlab_proto::ViewerReply::Pong) => {}
        _ => return Ok(false),
    }
    let fig_id = next_fig_id();
    VIEWER_CONN.with(|c| *c.borrow_mut() = Some(ViewerConn {
        client,
        fig_id,
        layout: (0, 0), // forces FigureOpen on first sync
    }));
    Ok(true)
}

/// Disconnect from the viewer and return to TUI mode.
/// Closes all viewer figures.
pub fn disconnect_viewer() {
    VIEWER_CONN.with(|c| {
        if let Some(mut conn) = c.borrow_mut().take() {
            let _ = conn.client.send_nowait(&ViewerMsg::Close { fig_id: conn.fig_id });
        }
    });
}

/// Start a new figure in the viewer, keeping the previous one visible.
/// No-op if the viewer is not connected.
pub fn viewer_new_figure() {
    VIEWER_CONN.with(|c| {
        let mut guard = c.borrow_mut();
        if let Some(ref mut conn) = *guard {
            conn.fig_id = next_fig_id();
            conn.layout = (0, 0); // forces FigureOpen on next sync
        }
    });
}

/// Returns true when a viewer connection is active for regular plotting.
pub fn viewer_active() -> bool {
    VIEWER_CONN.with(|c| c.borrow().is_some())
}

/// Get the current viewer figure ID, if connected.
pub fn get_viewer_fig_id() -> Option<u32> {
    VIEWER_CONN.with(|c| c.borrow().as_ref().map(|conn| conn.fig_id))
}

/// Set the viewer figure ID (for figure switching). Resets layout to force FigureOpen.
pub fn set_viewer_fig_id(id: u32) {
    VIEWER_CONN.with(|c| {
        if let Some(ref mut conn) = *c.borrow_mut() {
            conn.fig_id = id;
            conn.layout = (0, 0);
        }
    });
}

/// Allocate a new viewer figure ID without changing VIEWER_CONN state.
pub fn allocate_viewer_fig_id() -> u32 {
    next_fig_id()
}

/// Send the current FIGURE state to the viewer. No-op if not connected.
pub fn sync_viewer() {
    VIEWER_CONN.with(|c| {
        let mut guard = c.borrow_mut();
        let Some(ref mut conn) = *guard else { return; };
        FIGURE.with(|fig| {
            let fig = fig.borrow();
            let _ = send_figure_state(conn, &fig);
        });
    });
}

/// Serialize a full FigureState to the viewer via protocol messages.
fn send_figure_state(
    conn: &mut ViewerConn,
    fig: &FigureState,
) -> Result<(), PlotError> {
    let rows = fig.subplot_rows;
    let cols = fig.subplot_cols;
    let n_panels = rows * cols;
    let fig_id = conn.fig_id;

    // Only send FigureOpen when layout changes (or on first sync)
    if conn.layout != (rows, cols) {
        conn.client.send(&ViewerMsg::FigureOpen {
            id:    fig_id,
            rows:  rows as u16,
            cols:  cols as u16,
            title: String::new(),
        })?;
        conn.layout = (rows, cols);
    }

    for (idx, panel) in fig.subplots.iter().enumerate().take(n_panels) {
        // Convert series
        let wire_series: Vec<WireSeries> = panel.series.iter().enumerate().map(|(i, s)| {
            WireSeries {
                label: s.label.clone(),
                x: s.x_data.clone(),
                y: s.y_data.clone(),
                color: color_to_wire(&s.color),
                style: style_to_wire(&s.style),
                kind:  kind_to_wire(&s.kind),
                // Attach categorical labels to the first series
                x_labels: if i == 0 { panel.x_labels.clone() } else { None },
            }
        }).collect();

        conn.client.send_nowait(&ViewerMsg::PanelUpdate {
            fig_id,
            panel: idx as u16,
            series: wire_series,
        })?;

        conn.client.send_nowait(&ViewerMsg::PanelLabels {
            fig_id,
            panel: idx as u16,
            title:  panel.title.clone(),
            xlabel: panel.xlabel.clone(),
            ylabel: panel.ylabel.clone(),
        })?;

        conn.client.send_nowait(&ViewerMsg::PanelLimits {
            fig_id,
            panel: idx as u16,
            xlim: panel.xlim,
            ylim: panel.ylim,
        })?;
    }

    conn.client.send(&ViewerMsg::Redraw { fig_id })?;
    Ok(())
}
