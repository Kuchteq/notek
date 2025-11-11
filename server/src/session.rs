use std::io::{BufRead, Cursor, Read};

use algos::{doc::Doc, pid::Pid, session::SessionMessage, sync::DocOp};
use byteorder::{LittleEndian, ReadBytesExt};
use futures::stream::SplitSink;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{state::StateCommand};

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
        let req = SessionMessage::deserialize(&bin.to_vec());

    println!("{:#?}", req);
        match req {
            SessionMessage::Start {
                        document_id,
                        last_sync_time,
                    } => {
                        self.document_id = document_id;
                        self.connection_site_id = rand::rng().random_range(0..255);
                        state_tx.send(StateCommand::UpsertDoc {
                            document_id: self.document_id,
                        }).await;
                        println!("started a sesh");
                    }
            SessionMessage::Insert { site, pid, c } => {
                        let op = DocOp::Insert(pid, c);
                        state_tx.send(StateCommand::UpdateDoc {
                            document_id: self.document_id,
                            op,
                        }).await;
                    }
            SessionMessage::Delete { site, pid } => {
                        let op = DocOp::Delete(pid);
                        state_tx.send(StateCommand::UpdateDoc {
                            document_id: self.document_id,
                            op,
                        }).await;
                    }
            SessionMessage::NewSession { site, doc } => todo!(),
            SessionMessage::ChangeName { name } => {
                state_tx.send(StateCommand::ChangeName { document_id: self.document_id, name }).await;
            },
        }
        Ok(())
    }
    pub async fn flush_changes(&self, state_tx: &mpsc::Sender<StateCommand>) {
        state_tx.send(StateCommand::FlushChanges { document_id: self.document_id }).await;
    }

}
