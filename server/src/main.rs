use algos::Doc;
use std::fs;
use bincode::{config, DefaultOptions};

// fn main() {
//     let content = fs::read_to_string("foo.txt").unwrap();
//     let doc = Doc::new(content.to_string());
//     println!("{:#?}", doc.len());
// }
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9001").await?;
    println!("Listening on 127.0.0.1:9001");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    Ok(())
}

async fn handle_connection(stream: tokio::net::TcpStream) -> anyhow::Result<()> {
    let mut ws_stream = accept_async(stream).await?;
    println!("New WebSocket connection");

    let doc = Doc::new("Hello jego kurwa".to_string());

    let bytes = bincode::serialize(&doc).unwrap();
    let msg = Message::from(bytes);

    ws_stream.send(msg).await?;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await?; // Echo message
        }
    }

    Ok(())
}
