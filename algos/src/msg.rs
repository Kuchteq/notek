use serde::{Deserialize, Serialize};

use crate::{Doc, Pid};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PeerMessage {
    Greet,
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
    NewSessionRaw {
        site: u8,
        keys: Vec<Pid>,
        values: Vec<char>,
    },
}

impl PeerMessage {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            PeerMessage::Greet => vec![0u8],
            PeerMessage::NewSessionRaw { site, keys, values } => {
                let mut buf = vec![1u8]; 
                buf
            },
            PeerMessage::Insert { site, pid, c } => todo!(),
            PeerMessage::Delete { site, pid } => todo!(),
            PeerMessage::NewSession { site, doc } => todo!(),
        }
    }
}
