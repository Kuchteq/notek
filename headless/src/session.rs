use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use algos::session::SessionMessage;
use tungstenite::{connect, Message};

use crate::app::AppEvent;

const SERVER_URL: &str = "ws://127.0.0.1:9001";
const RETRY_INTERVAL: Duration = Duration::from_secs(5);

/// Sync thread: maintains a WebSocket connection to the sync server.
///
/// - On startup (and after disconnection), attempts to connect in a loop
///   with a delay between attempts.
/// - Signals `ServerConnected` / `ServerDisconnected` to the app event loop.
/// - Drains `SyncRequests` from `rx` and sends them over the WebSocket.
/// - If the WebSocket send fails, signals disconnection and reconnects.
pub fn handle_session_communication(rx: mpsc::Receiver<SessionMessage>, app_tx: mpsc::Sender<AppEvent>) {
    loop {
        // --- connect phase: retry until we get a connection ---
        let mut ws = loop {
            match connect(SERVER_URL) {
                Ok((ws, _)) => {
                    println!("Session: connected to {}", SERVER_URL);
                    let _ = app_tx.send(AppEvent::SessionConnected);
                    break ws;
                }
                Err(e) => {
                    println!(
                        "Session: connection failed ({}), retrying in {:?}...",
                        e, RETRY_INTERVAL
                    );
                    // Drain any messages that arrived while we were disconnected
                    // so we don't build up an unbounded backlog.
                    // They will be re-synced on next successful connection anyway.
                    drain(&rx);
                    thread::sleep(RETRY_INTERVAL);
                }
            }
        };

        // --- send phase: forward messages until the channel closes or WS breaks ---
        loop {
            match rx.recv() {
                Ok(cmd) => {
                    let msg = Message::from(cmd.serialize());
                    if let Err(e) = ws.send(msg) {
                        eprintln!("Session: send failed ({}), reconnecting...", e);
                        let _ = app_tx.send(AppEvent::SessionDisconnected);
                        break; // back to connect phase
                    }
                }
                Err(_) => {
                    // Channel closed — app is shutting down
                    let _ = ws.close(None);
                    return;
                }
            }
        }
    }
}

/// Drain all pending messages from the receiver without blocking.
fn drain(rx: &mpsc::Receiver<SessionMessage>) {
    loop {
        match rx.try_recv() {
            Ok(_) => continue,
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
        }
    }
}
