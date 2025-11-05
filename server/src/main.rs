use anyhow::{anyhow};
use futures::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::session::{SessionMember};
use crate::state::{State, StateCommand};
use crate::sync::{SyncRequests};
mod session;
mod state;
mod sync;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    println!("Listening on 0.0.0.0:9001");

    let (tx, rx) = mpsc::channel(100); // shared channel to state manager

    tokio::spawn(async {
        let mut state = State::init("./").unwrap();
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
    let mut ws = accept_async(stream).await?;

    if let Some(msg) = ws.next().await {
        let msg = msg?;
        if let Message::Binary(bin) = msg {
            match bin[0] {
                0..63 => {
                    start_handling_sync_requests(bin.to_vec(), state_tx, ws).await;
                }
                _ => {
                    start_handling_session_requests(bin.to_vec(), state_tx, ws).await;
                }
            }
        }
    }
    Ok(())
}

async fn start_handling_sync_requests(
    first_bin: Vec<u8>,
    state_tx: mpsc::Sender<StateCommand>,
    ws: WebSocketStream<TcpStream>,
) -> anyhow::Result<()> {
    let (mut ws_sink, mut ws_stream) = ws.split();
    handle_sync_request(first_bin, &state_tx, &mut ws_sink).await;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if let Message::Binary(bin) = msg {
            handle_sync_request(bin.to_vec(), &state_tx, &mut ws_sink).await;
        }
    }

    Ok(())
}

async fn start_handling_session_requests(
    first_bin: Vec<u8>,
    state_tx: mpsc::Sender<StateCommand>,
    ws: WebSocketStream<TcpStream>,
) -> anyhow::Result<()> {
    let (mut ws_sink, mut ws_stream) = ws.split();
    if first_bin[0] != 64 {
        anyhow!("First session message should be a start!");
    }
    let mut session = SessionMember::init();
    session
        .handle_session_request(first_bin, &state_tx, &mut ws_sink)
        .await?;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if let Message::Binary(bin) = msg {
            session
                .handle_session_request(bin.to_vec(), &state_tx, &mut ws_sink)
                .await?;
        }
    }
    Ok(())
}
async fn handle_sync_request(
    bin: Vec<u8>,
    state_tx: &mpsc::Sender<StateCommand>,
    ws_sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> anyhow::Result<()> {
    let req = SyncRequests::deserialize(bin);

    println!("{:#?}", req);
    match req {
        SyncRequests::SyncList { .. } => {
            let (resp_tx, resp_rx) = oneshot::channel();
            state_tx
                .send(StateCommand::GetSyncList {
                    last_sync_time: 0,
                    respond_to: resp_tx,
                })
                .await?;
            let buf = resp_rx.await?;
            ws_sink.send(Message::from(buf)).await?;
        }
        SyncRequests::SyncDoc { document_id, .. } => {
            let (resp_tx, resp_rx) = oneshot::channel();
            state_tx
                .send(StateCommand::GetSyncFullDoc {
                    document_id,
                    respond_to: resp_tx,
                })
                .await?;
            let buf = resp_rx.await?;
            ws_sink.send(Message::from(buf)).await?;
        }
    }
    Ok(())
}
