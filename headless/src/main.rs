use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::io::{Read, Write};
use std::{fs, thread};
use std::sync::mpsc;

use algos::doc::Doc;
use algos::session::SessionMessage;
use tungstenite::{connect, Message};

use crate::editor_message::EditorMessage;

mod editor_message;

fn handle_server_communication (rx: mpsc::Receiver<SessionMessage>) {
    let (mut ws, _) = connect("ws://127.0.0.1:9001").unwrap();

    while let Ok(cmd) = rx.recv() {
        let msg = Message::from(cmd.serialize());
        ws.send(msg);
    };
    
}



fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/editor_socket.sock";

    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    println!("Server listening on {}", socket_path);
    let (tx, rx) = mpsc::channel::<SessionMessage>();

    let d = Doc::new("Hello world! This app is the best thing ever!");

    thread::spawn(move || {
        handle_server_communication(rx);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let updates = EditorMessage::deserialize_all(stream);

                for update in updates.unwrap() {
                    match update {
                        EditorMessage::Insert(idx, chr) => {
                            
                        }
                        EditorMessage::Delete(idx) => todo!(),
                    }
                }


                // stream.write_all(b"Hello from server")?;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }

    Ok(())
}
