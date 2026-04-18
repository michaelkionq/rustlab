//! rustlab-viewer — standalone interactive plot viewer for rustlab.
//!
//! Listens on a Unix socket for plot data from `rustlab` and renders it
//! using egui with zoom, pan, crosshairs, and point readout.
//!
//! Usage:
//!     rustlab-viewer                 # default socket path
//!     rustlab-viewer --socket PATH   # custom socket path
//!     rustlab-viewer --name work     # named session (separate socket)

mod app;
mod figure;
mod net;
mod render;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("rustlab-viewer {}", env!("CARGO_PKG_VERSION"));
        println!("Standalone interactive plot viewer for rustlab\n");
        println!("Usage: rustlab-viewer [OPTIONS]\n");
        println!("Options:");
        println!("  --name NAME    Named session (connect with `viewer on NAME`)");
        println!("  --socket PATH  Custom Unix socket path (overrides --name)");
        println!("  -h, --help     Print help");
        println!("  -V, --version  Print version");
        return;
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("rustlab-viewer {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Parse optional --socket argument (takes precedence)
    if let Some(pos) = args.iter().position(|a| a == "--socket") {
        if let Some(path) = args.get(pos + 1) {
            std::env::set_var("RUSTLAB_VIEWER_SOCK", path);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "--name") {
        if let Some(name) = args.get(pos + 1) {
            let path = rustlab_proto::socket_path_for_name(name);
            std::env::set_var("RUSTLAB_VIEWER_SOCK", path);
        }
    }

    // Resolve the window title from the session name
    let title = if let Some(pos) = args.iter().position(|a| a == "--name") {
        args.get(pos + 1)
            .map(|n| format!("RustLab Viewer — {}", n))
            .unwrap_or_else(|| "RustLab Viewer".to_string())
    } else {
        "RustLab Viewer".to_string()
    };

    // Start socket listener in background
    let rx = net::start_listener();

    // Launch eframe GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |_cc| Ok(Box::new(app::ViewerApp::new(rx)))),
    )
    .expect("failed to run eframe");

    // Clean up socket on exit
    let sock = rustlab_proto::default_socket_path();
    let _ = std::fs::remove_file(&sock);
}
