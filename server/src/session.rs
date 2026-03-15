use algos::{pid::Pid, session::SessionMessage, sync::DocOp};
use futures::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use rand::Rng;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use anyhow::anyhow;
use crate::state::StateCommand;

pub async fn start_handling_session_requests(
    first_bin: Vec<u8>,
    state_tx: mpsc::Sender<StateCommand>,
    mut ws_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut ws_stream: SplitStream<WebSocketStream<TcpStream>>,
) -> anyhow::Result<()> {
    if first_bin[0] != 64 {
        return Err(anyhow!("First session message should be a start!"));
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
    println!("Finished");
    session.flush_changes(&state_tx).await;
    Ok(())
}

pub struct SessionMember {
    document_id: u128,
    connection_site_id: u8,
}

impl SessionMember {
    pub fn init() -> Self {
        SessionMember {
            connection_site_id: 0,
            document_id: 0,
        }
    }
    pub async fn handle_session_request(
        &mut self,
        bin: Vec<u8>,
        state_tx: &mpsc::Sender<StateCommand>,
        ws_sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> anyhow::Result<()> {
        let req = SessionMessage::deserialize(&bin);

        println!("{:#?}", req);
        match req {
            SessionMessage::Start {
                document_id,
                last_sync_time,
                name,
            } => {
                if self.document_id != 0 {
                    let _ = state_tx
                        .send(StateCommand::FlushChanges {
                            document_id: self.document_id,
                        })
                        .await;
                }
                self.document_id = document_id;
                self.connection_site_id = rand::rng().random_range(0..255);
                if let Some(name) = name {
                    let _ = state_tx
                        .send(StateCommand::UpsertDoc {
                            document_id: self.document_id,
                            name,
                        })
                        .await;
                }
                println!("started a sesh");
            }
            SessionMessage::Insert { site, pid, c } => {
                let op = DocOp::Insert(pid, c);
                let _ = state_tx
                    .send(StateCommand::UpdateDoc {
                        document_id: self.document_id,
                        op,
                    })
                    .await;
            }
            SessionMessage::Delete { site, pid } => {
                let op = DocOp::Delete(pid);
                let _ = state_tx
                    .send(StateCommand::UpdateDoc {
                        document_id: self.document_id,
                        op,
                    })
                    .await;
            }
            SessionMessage::ChangeName { name } => {
                let _ = state_tx
                    .send(StateCommand::ChangeName {
                        document_id: self.document_id,
                        name,
                    })
                    .await;
            }
        }
        Ok(())
    }
    pub async fn flush_changes(&self, state_tx: &mpsc::Sender<StateCommand>) {
        let _ = state_tx
            .send(StateCommand::FlushChanges {
                document_id: self.document_id,
            })
            .await;
    }
}
