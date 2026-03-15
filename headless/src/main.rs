use std::env;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process;
use std::sync::mpsc::{self, Sender};
use std::{fs, thread};

use algos::session::SessionMessage;
use algos::structure::DocStructure;
use algos::sync::SyncRequests;
use tungstenite::{connect, Message};

use crate::app::{run_app, AppEvent};
use crate::editor_message::EditorMessage;
use crate::monitor::monitor_updates;
use crate::state::State;
use crate::sync::handle_sync_communication;

mod app;
mod editor_message;
mod monitor;
mod state;
mod sync;

fn handle_server_communication(rx: mpsc::Receiver<SessionMessage>) {
    let (mut ws, _) = connect("ws://127.0.0.1:9001").unwrap();
    while let Ok(cmd) = rx.recv() {
        let msg = Message::from(cmd.serialize());
        let _ = ws.send(msg);
    }
}

fn accept_connections(listener: UnixListener, tx: Sender<AppEvent>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tx = tx.clone();
                thread::spawn(move || {
                    handle_client(stream, tx);
                });
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }
}

fn handle_client(mut stream: UnixStream, tx: Sender<AppEvent>) {
    loop {
        match EditorMessage::deserialize(&mut stream) {
            Ok(msg) => {
                if tx.send(AppEvent::EditorMsg(msg)).is_err() {
                    break;
                }
            }
            Err(_) => {
                let _ = tx.send(AppEvent::ClientDisconnected);
                break;
            }
        }
    }
}

fn read_doc(path: &str) {
    let path = PathBuf::from(path);

    if !path.exists() {
        eprintln!("File not found: {:?}", path);
        process::exit(1);
    }

    match DocStructure::read_existing(&path, &path) {
        Ok(doc) => {
            print!("{}", doc.get_doc().to_string());
        }
        Err(e) => {
            eprintln!("Failed to read structure file: {}", e);
            process::exit(1);
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle -r flag: read and print document contents, then exit
    if args.len() >= 3 && args[1] == "-r" {
        read_doc(&args[2]);
        return Ok(());
    }

    let socket_path = "/tmp/editor_socket.sock";

    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    println!("Server listening on {}", socket_path);

    let (session_tx, session_rx) = mpsc::channel::<SessionMessage>();
    let (sync_tx, sync_rx) = mpsc::channel::<SyncRequests>();
    let (tx, rx) = mpsc::channel::<AppEvent>();

    let mut state = State::init(PathBuf::from("./").as_path()).unwrap();

    thread::spawn(move || {
        handle_server_communication(session_rx);
    });

    let sync_app_tx = tx.clone();
    thread::spawn(move || {
        handle_sync_communication(sync_rx, sync_app_tx);
    });

    let base_dir = state.base_dir.clone();
    let inotify_tx = tx.clone();
    thread::spawn(move || {
        monitor_updates(inotify_tx, &base_dir);
    });

    let accept_tx = tx.clone();
    thread::spawn(move || {
        accept_connections(listener, accept_tx);
    });

    run_app(rx, &mut state, session_tx, sync_tx);

    Ok(())
}
