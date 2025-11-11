use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::io::{Read, Write};
use std::fs;

use tungstenite::connect;

fn handle_server_communication () {
    let (ws, _) = connect("ws://127.0.0.1:9001").unwrap();
}

fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/rust_socket.sock";

    // Remove any existing socket file
    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path)?;
    }

    // Bind to the socket
    let listener = UnixListener::bind(socket_path)?;
    println!("Server listening on {}", socket_path);


    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Client connected!");
                let mut buffer = [0u8; 1024];
                let n = stream.read(&mut buffer)?;
                println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

                stream.write_all(b"Hello from server")?;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }

    Ok(())
}
