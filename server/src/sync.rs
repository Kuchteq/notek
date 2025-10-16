use std::{io::Cursor, panic};

use algos::pid::Pid;
use byteorder::{LittleEndian, ReadBytesExt};

pub enum SyncRequests {
    // epoch miliseconds u64
    SyncList {
        last_sync_time: u64,
    },
    SyncDoc {
        last_sync_time: u64,
        document_id: u128,
    },
}

impl SyncRequests {
    pub fn deserialize(buf: Vec<u8>) -> Self {
        let mut cur = Cursor::new(buf);
        match cur.read_u8().unwrap() {
            0u8 => SyncRequests::SyncList {
                last_sync_time: cur.read_u64::<LittleEndian>().unwrap(),
            },
            1u8 => SyncRequests::SyncDoc {
                last_sync_time: cur.read_u64::<LittleEndian>().unwrap(),
                document_id: cur.read_u128::<LittleEndian>().unwrap(),
            },
            _ => panic!(),
        }
    }
}

pub enum SyncResponses {
    SyncList(Vec<DocSyncInfo>),
    SyncDoc {
        document_id: u128,
        updates: Vec<DocOp>,
    },
}

pub struct DocSyncInfo {
    last_mod_time: u64,
    document_id: u128,
}

impl DocSyncInfo {
    pub fn new(last_mod_time: u64, document_id: u128) -> Self {
        Self { last_mod_time, document_id }
    }
}

impl SyncResponses {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            SyncResponses::SyncList(doc_sync_infos) => {
                let mut buf = vec![2u8];
                buf.extend((doc_sync_infos.len() as u64).to_le_bytes());
                for doc in doc_sync_infos {
                    buf.extend(doc.last_mod_time.to_le_bytes());
                    buf.extend(doc.document_id.to_le_bytes());
                }
                buf
            }
            SyncResponses::SyncDoc {
                document_id,
                updates,
            } => {
                let mut buf = vec![3u8];
                buf.extend(document_id.to_le_bytes());
                buf.push(0u8); // hardcode for now as the whole document sync will come later
                buf.extend(updates.len().to_le_bytes());
                for op in updates {
                    op.serialize_into(&mut buf);
                }
                buf
            }
        }
    }
}

enum DocOp {
    Insert(Pid, char),
    Delete(Pid),
}

impl DocOp {
    pub fn serialize_into(&self, buf: &mut Vec<u8>) {
        match self {
            DocOp::Insert(pid, char) => {
                buf.push(0);
                let mut cbuf = [0u8; 4];
                let encoded = char.encode_utf8(&mut cbuf);
                // put atom's data length
                buf.push(encoded.len() as u8);
                buf.extend(encoded.as_bytes());
                pid.write_bytes(buf);
            }
            DocOp::Delete(pid) => {
                buf.push(1);
                buf.push(pid.0.len() as u8);
                pid.write_bytes(buf);
            }
        }
    }
}
