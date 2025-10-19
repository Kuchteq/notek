use std::io::{Cursor, Read};

use algos::{doc::Doc, pid::Pid};
use byteorder::{LittleEndian, ReadBytesExt};
use futures::stream::SplitSink;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{state::StateCommand, sync::DocOp};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SessionMessage {
    Start {
        document_id: u128,
        last_sync_time: u64,
    },
    Insert {
        site: u8,
        pid: Pid,
        c: char,
    },
    Delete {
        site: u8,
        pid: Pid,
    },
    NewSession {
        site: u8,
        doc: Doc,
    },
}

impl SessionMessage {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            SessionMessage::Start {
                document_id,
                last_sync_time,
            } => vec![0u8],
            SessionMessage::NewSession { site, doc } => {
                // put header i.e. PeerMessage enum it is
                let mut buf = vec![1u8];
                // put site_id
                buf.push(*site);
                // put numberofatoms
                buf.extend((doc.len() as u64).to_le_bytes());
                doc.write_bytes_tobuf(&mut buf);
                buf
            }
            SessionMessage::Insert { site, pid, c } => {
                let mut buf = vec![2u8];
                buf.push(*site);
                let mut cbuf = [0u8; 4];
                let encoded = c.encode_utf8(&mut cbuf);
                // put atom's data length
                buf.push(encoded.len() as u8);
                buf.extend(encoded.as_bytes());
                buf.push(pid.depth() as u8);
                pid.write_bytes(&mut buf);
                buf
            }
            SessionMessage::Delete { site, pid } => {
                let mut buf = vec![3u8];
                buf.push(*site);
                buf.push(pid.depth() as u8);
                pid.write_bytes(&mut buf);
                buf
            }
        }
    }
    pub fn deserialize(buf: &[u8]) -> SessionMessage {
        let mut cur = Cursor::new(buf);
        match cur.read_u8().unwrap() {
            64u8 => {
                let last_sync_time = cur.read_u64::<LittleEndian>().unwrap();
                let document_id = cur.read_u128::<LittleEndian>().unwrap();
                SessionMessage::Start {
                    document_id,
                    last_sync_time,
                }
            },
            65u8 => {
                let site = cur.read_u8().unwrap();
                let number_of_atoms = cur.read_u64::<LittleEndian>().unwrap() as usize;
                SessionMessage::NewSession {
                    site: site,
                    doc: Doc::from_reader(&mut cur, number_of_atoms),
                }
            }
            66u8 => {
                let site = cur.read_u8().unwrap();
                let data_len = cur.read_u8().unwrap() as usize;
                let mut bytes = [0u8; 4];
                cur.read_exact(&mut bytes[..data_len]).unwrap();
                let data = std::str::from_utf8(&bytes[..data_len])
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap();
                let pid_depth = cur.read_u8().unwrap();
                let pid = Pid::from_reader(&mut cur, pid_depth as usize);
                SessionMessage::Insert {
                    site: site,
                    pid: pid,
                    c: data,
                }
            }
            67u8 => {
                let site = cur.read_u8().unwrap();
                let pid_depth = cur.read_u8().unwrap();
                let pid = Pid::from_reader(&mut cur, pid_depth as usize);
                SessionMessage::Delete {
                    site: site,
                    pid: pid,
                }
            }
            _ => panic!(),
        }
    }
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
        let req = SessionMessage::deserialize(&bin.to_vec());

        match req {
            SessionMessage::Start {
                document_id,
                last_sync_time,
            } => {
                self.document_id = document_id;
                self.connection_site_id = rand::rng().random_range(0..255);
            }
            SessionMessage::Insert { site, pid, c } => {
                let op = DocOp::Insert(pid, c);
                state_tx.send(StateCommand::UpdateDoc {
                    document_id: self.document_id,
                    op,
                });
            }
            SessionMessage::Delete { site, pid } => {
                let op = DocOp::Delete(pid);
                state_tx.send(StateCommand::UpdateDoc {
                    document_id: self.document_id,
                    op,
                });
            }
            SessionMessage::NewSession { site, doc } => todo!(),
        }
        Ok(())
    }
}
