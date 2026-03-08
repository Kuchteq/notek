use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{env, fs, thread};

use algos::doc::Doc;
use algos::session::SessionMessage;
use inotify::{EventMask, Inotify, WatchMask};
use tungstenite::{Message, connect};

use crate::editor_message::EditorMessage;
use crate::state::State;

mod editor_message;
mod state;

fn handle_server_communication(rx: mpsc::Receiver<SessionMessage>) {
    let (mut ws, _) = connect("ws://127.0.0.1:9001").unwrap();
    let start = SessionMessage::Start {
        document_id: u128::max_value(),
        last_sync_time: 0,
    };
    ws.send(Message::from(start.serialize()));
    while let Ok(cmd) = rx.recv() {
        let msg = Message::from(cmd.serialize());
        ws.send(msg);
    }
}

fn monitor_updates() {
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");

    let current_dir = env::current_dir().expect("Failed to determine current directory");

    inotify
        .watches()
        .add(&current_dir, WatchMask::MOVE)
        .expect("Failed to add inotify watch");

    println!("Watching current directory for activity...");

    let mut pending_moves: HashMap<u32, PathBuf> = HashMap::new();
    let mut buffer = [0u8; 4096];

    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        for event in events {
            let name = match event.name {
                Some(n) => current_dir.join(n),
                None => continue,
            };

            if event.mask.contains(EventMask::MOVED_FROM) {
                pending_moves.insert(event.cookie, name.clone());
            }

            if event.mask.contains(EventMask::MOVED_TO) {
                if let Some(src) = pending_moves.remove(&event.cookie) {
                    println!("Renamed: {:?} -> {:?}", src, name);
                } else {
                    println!("File moved into watched dir: {:?}", name);
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/editor_socket.sock";

    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    println!("Server listening on {}", socket_path);
    let (tx, rx) = mpsc::channel::<SessionMessage>();

    thread::spawn(move || {
        handle_server_communication(rx);
    });

    thread::spawn(move || {
        monitor_updates();
    });

    let mut state = State::init(PathBuf::from("./").as_path()).unwrap();
    // println!("{}",d.to_string());
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                loop {
                    let message = EditorMessage::deserialize(&mut stream);
                    match message {
                        Ok(EditorMessage::ChooseDocument(doc_name)) => {
                            // println!("Document chosen: {}", doc_name);
                            state.set_current_doc(&doc_name);
                        }
                        Ok(EditorMessage::Insert(pos, text)) => {
                            println!("Text received {} {}", pos, text);
                            state.insert_in_current_doc(pos, &text);
                            // for (pid, c) in inserted {
                            //     let msg = SessionMessage::Insert { site: 0, pid, c };
                            //     tx.send(msg);
                            // }
                        }
                        Ok(EditorMessage::Delete(start, len)) => {
                            println!("Text deleted from range: {} {}", start, len);
                            state.delete_in_current_doc(start, len);
                            // for pid in deleted {
                            //     let msg = SessionMessage::Delete { site: 0, pid };
                            //     tx.send(msg);
                            // }
                        }
                        Ok(EditorMessage::Flush) => {
                            state.flush_current_doc();
                        }
                        Err(_) => break,
                    }
                    // stream.write_all(b"Hello from server")?;
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }

    Ok(())
}
