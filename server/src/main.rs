use algos::doc::Doc;
use algos::msg::PeerMessage;
use algos::pid::Pid;
use rand::Rng;
use std::{fs, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::serializer::Serializer;
mod serializer;
mod sync;

enum DocCommand {
    Insert(u8, Pid, char),
    Delete(u8, Pid),
    GetSnapshot(oneshot::Sender<Arc<Doc>>),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    println!("Listening on 127.0.0.1:9001");

    let (dcmd_tx, mut dcmd_rx) = mpsc::unbounded_channel::<DocCommand>();
    let (bcast_tx, _) = broadcast::channel::<PeerMessage>(100);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(
            stream,
        ));
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
) -> anyhow::Result<()> {
    let ws = accept_async(stream).await?;
    let connection_site_id = rand::rng().random_range(0..255);
    let (mut ws_sink, mut ws_stream) = ws.split();
    let serializer = Serializer::new(serializer::SerializeFormat::Mine);

    println!(
        "New WebSocket connection, assigned id: {}",
        connection_site_id
    );

    loop {
        tokio::select! {
            Some(msg) = ws_stream.next() => {
                    let msg = msg?;
                    if let Message::Binary(bin) = msg {

                    }
            }
        }
    }

    Ok(())
}
