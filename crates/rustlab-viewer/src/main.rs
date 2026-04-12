//! rustlab-viewer — standalone interactive plot viewer for rustlab.
//!
//! Listens on a Unix socket for plot data from `rustlab` and renders it
//! using egui with zoom, pan, crosshairs, and point readout.
//!
//! Usage:
//!     rustlab-viewer                 # default socket path
//!     rustlab-viewer --socket PATH   # custom socket path

mod app;
mod figure;
mod net;
mod render;

fn main() {
    // Parse optional --socket argument
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--socket") {
        if let Some(path) = args.get(pos + 1) {
            std::env::set_var("RUSTLAB_VIEWER_SOCK", path);
        }
    }

    // Start socket listener in background
    let rx = net::start_listener();

    // Launch eframe GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RustLab Viewer")
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RustLab Viewer",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(app::ViewerApp::new(rx)))
        }),
    )
    .expect("failed to run eframe");

    // Clean up socket on exit
    let sock = rustlab_proto::default_socket_path();
    let _ = std::fs::remove_file(&sock);
}
