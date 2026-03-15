use algos::sync::{DocOp, SyncRequests};
use futures::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use std::io::Cursor;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::state::StateCommand;

pub async fn start_handling_sync_requests(
    first_bin: Vec<u8>,
    state_tx: mpsc::Sender<StateCommand>,
    mut ws_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut ws_stream: SplitStream<WebSocketStream<TcpStream>>,
) -> anyhow::Result<()> {
    handle_sync_request(first_bin, &state_tx, &mut ws_sink).await?;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if let Message::Binary(bin) = msg {
            handle_sync_request(bin.to_vec(), &state_tx, &mut ws_sink).await?;
        }
    }
    Ok(())
}

async fn handle_sync_request(
    bin: Vec<u8>,
    state_tx: &mpsc::Sender<StateCommand>,
    ws_sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> anyhow::Result<()> {
    let req = SyncRequests::deserialize(Cursor::new(&bin))?;

    println!("{:#?}", req);
    match req {
        SyncRequests::SyncList { last_sync_time } => {
            let (resp_tx, resp_rx) = oneshot::channel();
            state_tx
                .send(StateCommand::GetSyncList {
                    last_sync_time,
                    respond_to: resp_tx,
                })
                .await?;
            let buf = resp_rx.await?;
            ws_sink.send(Message::from(buf)).await?;
        }
        SyncRequests::SyncDoc {
            document_id,
            last_sync_time,
        } => {
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
        SyncRequests::SyncDocUpsert {
            document_id,
            name,
            last_sync_time: _,
            inserts,
            deletes,
        } => {
            // Upsert the document (create if missing, or update name)
            if let Some(name) = name {
                state_tx
                    .send(StateCommand::UpsertDoc {
                        document_id,
                        name,
                    })
                    .await?;
            }

            // Apply all inserts
            for (pid, ch) in inserts {
                state_tx
                    .send(StateCommand::UpdateDoc {
                        document_id,
                        op: DocOp::Insert(pid, ch),
                    })
                    .await?;
            }

            // Apply all deletes
            for pid in deletes {
                state_tx
                    .send(StateCommand::UpdateDoc {
                        document_id,
                        op: DocOp::Delete(pid),
                    })
                    .await?;
            }

            // Flush after applying all changes
            state_tx
                .send(StateCommand::FlushChanges { document_id })
                .await?;
        }
        SyncRequests::DocNameChange { document_id, name } => {
            state_tx
                .send(StateCommand::ChangeName {
                    document_id,
                    name,
                })
                .await?;
        }
        SyncRequests::DeleteDoc { document_id } => {
            state_tx
                .send(StateCommand::DeleteDoc { document_id })
                .await?;
        }
    }
    Ok(())
}
