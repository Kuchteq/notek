use algos::doc::Doc;
use algos::msg::PeerMessage;
use algos::pid::Pid;
use futures::stream::SplitSink;
use rand::Rng;
use tokio::task::LocalSet;
use std::{fs, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Bytes;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::serializer::Serializer;
use crate::session::SessionMessage;
use crate::state::{State, StateCommand};
use crate::sync::{DocSyncInfo, SyncRequests, SyncResponses};
mod serializer;
mod state;
mod sync;
mod session;

enum DocCommand {
    Insert(u8, Pid, char),
    Delete(u8, Pid),
    GetSnapshot(oneshot::Sender<Arc<Doc>>),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    println!("Listening on 0.0.0.0:9001");

    let (tx, rx) = mpsc::channel(100);

    // your state manager is NOT Send — so it goes on a LocalSet
    let local = LocalSet::new();
    let state_task = async move {
        let mut state = State::init("sample")?;
        state.run_state_manager(rx).await;
        Ok::<_, anyhow::Error>(())
    };

    // spawn your actor locally (not Send!)
    local.spawn_local(state_task);

    // Run LocalSet alongside your normal async I/O tasks
    local
        .run_until(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(handle_connection(stream, tx.clone()));
            }
        })
        .await;

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    state_tx: mpsc::Sender<StateCommand>,
) -> anyhow::Result<()> {
    let mut ws = accept_async(stream).await?;
    let connection_site_id = rand::rng().random_range(0..255);

    println!(
        "New WebSocket connection, assigned id: {}",
        connection_site_id
    );

    if let Some(msg) = ws.next().await {
        let msg = msg?;
        if let Message::Binary(bin) = msg {
            match bin[0] {
                0..63 => {
                    start_handling_sync_requests(bin.to_vec(), state_tx, ws).await;
                }
                _ => { start_handling_session_requests(bin.to_vec(), state_tx, ws).await; }
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
        
    Ok(())
}
async fn handle_sync_request(
    bin: Vec<u8>,
    state_tx: &mpsc::Sender<StateCommand>,
    ws_sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> anyhow::Result<()> {
    let req = SyncRequests::deserialize(bin);
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
