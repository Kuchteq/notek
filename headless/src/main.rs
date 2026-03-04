use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::{fs, thread};

use algos::doc::Doc;
use algos::session::SessionMessage;
use tungstenite::{Message, connect};

use crate::editor_message::EditorMessage;
use crate::local_doc::LocalDoc;

mod editor_message;
mod local_doc;

fn handle_server_communication(rx: mpsc::Receiver<SessionMessage>) {
    let (mut ws, _) = connect("ws://127.0.0.1:9001").unwrap();

    while let Ok(cmd) = rx.recv() {
        let msg = Message::from(cmd.serialize());
        ws.send(msg);
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

    let mut d = LocalDoc::default();

    thread::spawn(move || {
        handle_server_communication(rx);
    });

    // println!("{}",d.to_string());
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                loop {
                    let message = EditorMessage::deserialize(&mut stream);
                    match message {
                        Ok(EditorMessage::Insert(idx, text)) => {
                            println!("Text received {} {}", idx, text);
                            d.insert_at_byte(idx as usize, text);
                        }
                        Ok(EditorMessage::Delete(start, len)) => {
                            println!("Text deleted from range: {} {}", start, len);
                            d.delete_bytes(start as usize, len as usize)
                        }
                        Err(_) => break,
                    }
                    println!("{}",d.crdt.to_abs_string())
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
