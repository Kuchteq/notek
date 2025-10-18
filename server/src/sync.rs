use std::{
    io::{Cursor, Write},
    panic,
};

use algos::{doc::Doc, pid::Pid};
use anyhow::Result;
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

pub enum SyncResponses<'a> {
    SyncList(Vec<DocSyncInfo>),
    SyncOpDoc {
        document_id: u128,
        updates: Vec<DocOp>,
    },
    SyncFullDoc {
        document_id: u128,
        doc: &'a Doc,
    },
}

pub struct DocSyncInfo {
    last_mod_time: u64,
    document_id: u128,
}

impl DocSyncInfo {
    pub fn new(last_mod_time: u64, document_id: u128) -> Self {
        Self {
            last_mod_time,
            document_id,
        }
    }
}

impl SyncResponses<'_> {
    pub fn serialize_into<W: Write>(&self, mut w: W) -> Result<()> {
        match self {
            SyncResponses::SyncList(doc_sync_infos) => {
                w.write_all(&[2u8])?;
                w.write_all(&(doc_sync_infos.len() as u64).to_le_bytes())?;
                for doc in doc_sync_infos {
                    w.write_all(&doc.last_mod_time.to_le_bytes())?;
                    w.write_all(&doc.document_id.to_le_bytes())?;
                }

            },
            SyncResponses::SyncOpDoc {
                document_id,
                updates,
            } => {
                w.write_all(&[3u8])?;
                w.write_all(&document_id.to_le_bytes())?;
                w.write_all(&(updates.len() as u64).to_le_bytes())?;

                for op in updates {
                    op.serialize_into(&mut w)?;
                }
            }
            SyncResponses::SyncFullDoc {
                document_id,
                doc,
            } => {
                w.write_all(&[4u8])?;
                w.write_all(&document_id.to_le_bytes())?;
                w.write_all(&(doc.len() as u64).to_le_bytes())?;
                doc.write_bytes(&mut w)?;
            },
        }
        Ok(())
    }
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
            SyncResponses::SyncOpDoc {
                document_id,
                updates,
            } => {
                let mut buf = vec![3u8];
                buf.extend(document_id.to_le_bytes());
                buf.extend(updates.len().to_le_bytes());
                for op in updates {
                    op.serialize_into(&mut buf);
                }
                buf
            }
            SyncResponses::SyncFullDoc {
                document_id,
                doc,
            } => todo!(),
        }
    }
}

enum DocOp {
    Insert(Pid, char),
    Delete(Pid),
}

impl DocOp {
    pub fn serialize_into<W: Write>(&self, mut w: W) -> Result<()> {
        match self {
            DocOp::Insert(pid, ch) => {
                w.write_all(&[0])?;

                // encode char as UTF-8
                let mut cbuf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut cbuf);

                // write length + bytes
                w.write_all(&[encoded.len() as u8])?;
                w.write_all(encoded.as_bytes())?;

                pid.write_bytes(&mut w)?;
            }

            DocOp::Delete(pid) => {
                w.write_all(&[1])?;
                w.write_all(&[pid.0.len() as u8])?;
                pid.write_bytes(&mut w)?;
            }
        }
        Ok(())
    }

    pub fn serialize(&self, buf: &mut Vec<u8>) {
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
