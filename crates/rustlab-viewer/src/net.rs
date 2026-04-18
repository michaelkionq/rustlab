//! Socket listener for incoming rustlab connections.

use rustlab_proto::{default_socket_path, read_msg, write_msg, ViewerMsg, ViewerReply};
use std::io::BufWriter;
use std::sync::mpsc;

/// Start the Unix socket listener in a background thread.
/// Returns a receiver for incoming messages.
pub fn start_listener() -> mpsc::Receiver<ViewerMsg> {
    let (tx, rx) = mpsc::channel();

    std::thread::Builder::new()
        .name("viewer-listener".into())
        .spawn(move || {
            if let Err(e) = run_listener(tx) {
                eprintln!("rustlab-viewer: listener error: {}", e);
            }
        })
        .expect("failed to spawn listener thread");

    rx
}

fn run_listener(tx: mpsc::Sender<ViewerMsg>) -> std::io::Result<()> {
    let path = default_socket_path();

    // Check for existing socket — if a live viewer is listening, refuse to start
    if path.exists() {
        #[cfg(unix)]
        {
            if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&path) {
                // Try a ping to see if it's alive
                if write_msg(&mut stream, &ViewerMsg::Ping).is_ok() {
                    if let Ok(Some(ViewerReply::Pong)) = read_msg::<_, ViewerReply>(&mut stream) {
                        eprintln!(
                            "rustlab-viewer: another viewer is already running on {}",
                            path.display()
                        );
                        eprintln!("  use --name <NAME> to start a separate session");
                        std::process::exit(1);
                    }
                }
            }
        }
        // Stale socket — remove it
        std::fs::remove_file(&path)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::net::UnixListener;

        let listener = UnixListener::bind(&path)?;
        eprintln!("rustlab-viewer: listening on {}", path.display());

        // Clean up socket on exit
        let path_clone = path.clone();
        ctrlc_cleanup(path_clone);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    eprintln!("rustlab-viewer: client connected");
                    let tx = tx.clone();
                    std::thread::Builder::new()
                        .name("viewer-conn".into())
                        .spawn(move || handle_connection(stream, tx))
                        .ok();
                }
                Err(e) => {
                    eprintln!("rustlab-viewer: accept error: {}", e);
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:19847")?;
        eprintln!("rustlab-viewer: listening on 127.0.0.1:19847");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    eprintln!("rustlab-viewer: client connected");
                    let tx = tx.clone();
                    std::thread::Builder::new()
                        .name("viewer-conn".into())
                        .spawn(move || handle_connection(stream, tx))
                        .ok();
                }
                Err(e) => {
                    eprintln!("rustlab-viewer: accept error: {}", e);
                }
            }
        }
    }

    Ok(())
}

fn handle_connection<S: std::io::Read + std::io::Write>(
    mut stream: S,
    tx: mpsc::Sender<ViewerMsg>,
) {
    loop {
        match read_msg::<_, ViewerMsg>(&mut stream) {
            Ok(Some(msg)) => {
                let is_ping = matches!(msg, ViewerMsg::Ping);
                let reply = if is_ping {
                    ViewerReply::Pong
                } else {
                    if tx.send(msg).is_err() {
                        return; // app shut down
                    }
                    ViewerReply::Ok
                };
                let mut bw = BufWriter::new(&mut stream);
                if write_msg(&mut bw, &reply).is_err() {
                    return;
                }
            }
            Ok(None) => return, // clean EOF
            Err(_) => return,   // broken pipe
        }
    }
}

#[cfg(unix)]
fn ctrlc_cleanup(path: std::path::PathBuf) {
    // Best-effort: remove socket on SIGINT/SIGTERM via atexit-like pattern.
    // The Drop-based cleanup in main is the primary mechanism.
    std::thread::Builder::new()
        .name("viewer-cleanup".into())
        .spawn(move || {
            // This thread just exists so the path is dropped on process exit
            // via the Drop guard below. We park it forever.
            let _guard = SocketCleanup(path);
            std::thread::park();
        })
        .ok();
}

#[cfg(unix)]
struct SocketCleanup(std::path::PathBuf);

#[cfg(unix)]
impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
