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
    let start = SessionMessage::Start { document_id: u128::max_value(), last_sync_time: 0 } ;
    ws.send(Message::from(start.serialize()));
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

    let mut d = Doc::default();

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
                        Ok(EditorMessage::Insert(pos, text)) => {
                            println!("Text received {} {}", pos, text);
                            let inserted = d.insert_text_at_bytepos(pos as usize, text);
                            for (pid, c) in inserted {
                                let msg = SessionMessage::Insert { site: 0, pid, c };
                                tx.send(msg);
                            }
                            
                        }
                        Ok(EditorMessage::Delete(start, len)) => {
                            println!("Text deleted from range: {} {}", start, len);
                            let deleted = d.delete_byte_range(start as usize, len as usize);
                            for pid in deleted {
                                let msg = SessionMessage::Delete { site: 0, pid };
                                tx.send(msg);
                            }
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
