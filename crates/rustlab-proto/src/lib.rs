//! Wire protocol for rustlab ↔ rustlab-viewer IPC.
//!
//! Messages are length-prefixed msgpack: `[u32 BE length][msgpack bytes]`.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

// ─── Wire types ─────────────────────────────────────────────────────────────

/// Message sent from rustlab (client) to rustlab-viewer (server).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ViewerMsg {
    /// Create or resize a figure window.
    FigureOpen {
        id:    u32,
        rows:  u16,
        cols:  u16,
        title: String,
    },
    /// Replace all series data in one panel (0-based).
    PanelUpdate {
        fig_id: u32,
        panel:  u16,
        series: Vec<WireSeries>,
    },
    /// Set axis labels and title for a panel.
    PanelLabels {
        fig_id: u32,
        panel:  u16,
        title:  String,
        xlabel: String,
        ylabel: String,
    },
    /// Set fixed axis limits for a panel.
    PanelLimits {
        fig_id: u32,
        panel:  u16,
        xlim:   (Option<f64>, Option<f64>),
        ylim:   (Option<f64>, Option<f64>),
    },
    /// Request a redraw of all panels in a figure.
    Redraw { fig_id: u32 },
    /// Close a figure window.
    Close { fig_id: u32 },
    /// Keepalive ping.
    Ping,
}

/// Response from rustlab-viewer back to rustlab.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ViewerReply {
    Ok,
    Error(String),
    Pong,
}

/// A single data series on the wire.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WireSeries {
    pub label: String,
    pub x:     Vec<f64>,
    pub y:     Vec<f64>,
    pub color: WireColor,
    pub style: WireLineStyle,
    pub kind:  WirePlotKind,
    /// Categorical x-axis tick labels (e.g. for bar charts with string categories).
    #[serde(default)]
    pub x_labels: Option<Vec<String>>,
}

/// Color on the wire.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WireColor {
    Named(String),
    Rgb(u8, u8, u8),
}

/// Line style on the wire.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WireLineStyle {
    Solid,
    Dashed,
}

/// Plot kind on the wire.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WirePlotKind {
    Line,
    Stem,
    Bar,
    Scatter,
}

// ─── Framing helpers ────────────────────────────────────────────────────────

/// Write a length-prefixed msgpack message.
pub fn write_msg<W: Write, T: Serialize>(w: &mut W, msg: &T) -> std::io::Result<()> {
    let payload = rmp_serde::to_vec(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let len = payload.len() as u32;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(&payload)?;
    w.flush()
}

/// Read a length-prefixed msgpack message.  Returns `None` on clean EOF.
pub fn read_msg<R: Read, T: for<'de> Deserialize<'de>>(r: &mut R) -> std::io::Result<Option<T>> {
    let mut len_buf = [0u8; 4];
    match r.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    let msg = rmp_serde::from_slice(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Some(msg))
}

// ─── Socket path ────────────────────────────────────────────────────────────

/// Default Unix socket path for the viewer.
///
/// Precedence: `$RUSTLAB_VIEWER_SOCK` > `/tmp/rustlab-viewer-{uid}.sock`
pub fn default_socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("RUSTLAB_VIEWER_SOCK") {
        return PathBuf::from(p);
    }
    #[cfg(unix)]
    {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/rustlab-viewer-{}.sock", uid))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from("/tmp/rustlab-viewer.sock")
    }
}

/// Socket path for a named viewer session.
///
/// Returns `/tmp/rustlab-viewer-{uid}-{name}.sock` (Unix) or
/// `/tmp/rustlab-viewer-{name}.sock` (non-Unix).
pub fn socket_path_for_name(name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/rustlab-viewer-{}-{}.sock", uid, name))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(format!("/tmp/rustlab-viewer-{}.sock", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_viewer_msg() {
        let msg = ViewerMsg::PanelUpdate {
            fig_id: 1,
            panel:  0,
            series: vec![WireSeries {
                label: "test".into(),
                x: vec![1.0, 2.0, 3.0],
                y: vec![4.0, 5.0, 6.0],
                color: WireColor::Named("cyan".into()),
                style: WireLineStyle::Solid,
                kind:  WirePlotKind::Line,
                x_labels: None,
            }],
        };
        let mut buf = Vec::new();
        write_msg(&mut buf, &msg).unwrap();
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded: Option<ViewerMsg> = read_msg(&mut cursor).unwrap();
        let decoded = decoded.expect("should decode");
        match decoded {
            ViewerMsg::PanelUpdate { fig_id, panel, series } => {
                assert_eq!(fig_id, 1);
                assert_eq!(panel, 0);
                assert_eq!(series.len(), 1);
                assert_eq!(series[0].x, vec![1.0, 2.0, 3.0]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn round_trip_reply() {
        let msg = ViewerReply::Error("test error".into());
        let mut buf = Vec::new();
        write_msg(&mut buf, &msg).unwrap();
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded: Option<ViewerReply> = read_msg(&mut cursor).unwrap();
        match decoded.unwrap() {
            ViewerReply::Error(s) => assert_eq!(s, "test error"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn eof_returns_none() {
        let buf: Vec<u8> = vec![];
        let mut cursor = std::io::Cursor::new(&buf);
        let result: Option<ViewerMsg> = read_msg(&mut cursor).unwrap();
        assert!(result.is_none());
    }
}
