use std::path::PathBuf;

use futures::StreamExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::session::start_handling_session_requests;
use crate::state::{State, StateCommand};
use crate::sync::start_handling_sync_requests;
mod session;
mod state;
mod sync;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    println!("Listening on 0.0.0.0:9001");

    let (tx, rx) = mpsc::channel(100); // shared channel to state manager
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        println!("Shutting down...");
        std::process::exit(0);
    });

    tokio::spawn(async {
        let mut state = State::init(PathBuf::from("./").as_path()).unwrap();
        state.run_state_manager(rx).await;
    });

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, tx.clone()));
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    state_tx: mpsc::Sender<StateCommand>,
) -> anyhow::Result<()> {
    let ws = accept_async(stream).await?;
    let (mut ws_sink, mut ws_stream) = ws.split();

    // Read the first message to determine connection type
    if let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if let Message::Binary(bin) = msg {
            match bin[0] {
                // Sync requests: first byte < 64 (tags 0-4)
                0..64 => {
                    start_handling_sync_requests(bin.to_vec(), state_tx, ws_sink, ws_stream)
                        .await?;
                }
                // Session requests: first byte >= 64 (tags 64+)
                _ => {
                    start_handling_session_requests(bin.to_vec(), state_tx, ws_sink, ws_stream)
                        .await?;
                }
            }
        }
    }
    Ok(())
}
