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
use crate::state::{State, StateCommand};
use crate::sync::{DocSyncInfo, SyncRequests, SyncResponses};
mod serializer;
mod state;
mod sync;

enum DocCommand {
    Insert(u8, Pid, char),
    Delete(u8, Pid),
    GetSnapshot(oneshot::Sender<Arc<Doc>>),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    println!("Listening on 0.0.0.0:9001");

    let (tx, rx) = mpsc::channel(100); // shared channel to state manager
    let mut state = State::init("sample").unwrap();
    tokio::spawn(async move { state.run_state_manager(rx).await; });

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
    let connection_site_id = rand::rng().random_range(0..255);
    let (mut ws_sink, mut ws_stream) = ws.split();

    println!("New WebSocket connection, assigned id: {}", connection_site_id);

    loop {
        if let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if let Message::Binary(bin) = msg {
                let req = SyncRequests::deserialize((&bin).to_vec());
                match req {
                    SyncRequests::SyncList { .. } => {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        state_tx .send(StateCommand::GetSyncList { last_sync_time: 0, respond_to: resp_tx })
                            .await?;
                        let buf = resp_rx.await?;
                        ws_sink.send(Message::from(buf)).await?;
                    }
                    SyncRequests::SyncDoc { document_id, .. } => {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        state_tx
                            .send(StateCommand::GetSyncFullDoc { document_id, respond_to: resp_tx })
                            .await?;
                        let buf = resp_rx.await?;
                        println!("{:#?}",buf);
                        ws_sink.send(Message::from(buf)).await?;
                    }
                }
            }
        }
    }
}
