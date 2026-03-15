use std::{
    io::{self, BufRead, Cursor, Read, Write},
    panic,
    path::PathBuf,
};

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{doc::Doc, pid::Pid};

#[derive(Debug)]
pub enum SyncRequests {
    SyncList {
        last_sync_time: u64,
    },
    SyncDoc {
        document_id: u128,
        last_sync_time: u64,
    },
    SyncDocUpsert {
        document_id: u128,
        name: Option<PathBuf>,
        last_sync_time: u64,
        inserts: Vec<(Pid, char)>,
        deletes: Vec<Pid>,
    },
    DocNameChange {
        document_id: u128,
        name: PathBuf,
    },
    DeleteDoc {
        document_id: u128,
    },
}


impl SyncRequests {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.serialize_into(&mut buf);
        buf
    }

    pub fn serialize_into<W: Write>(&self, mut w: W) -> io::Result<()> {
        match self {
            SyncRequests::SyncList { last_sync_time } => {
                w.write_u8(0)?;
                w.write_u64::<LittleEndian>(*last_sync_time)?;
            }

            SyncRequests::SyncDoc {
                document_id,
                last_sync_time,
            } => {
                w.write_u8(1)?;
                w.write_u128::<LittleEndian>(*document_id)?;
                w.write_u64::<LittleEndian>(*last_sync_time)?;
            }

            SyncRequests::SyncDocUpsert {
                document_id,
                name,
                last_sync_time,
                inserts,
                deletes,
            } => {
                w.write_u8(2)?;
                w.write_u128::<LittleEndian>(*document_id)?;

                match name {
                    Some(path) => {
                        w.write_all(path.to_string_lossy().as_bytes())?;
                        w.write_all(b"\n")?;
                    }
                    None => w.write_all(b"\n")?,
                }

                w.write_u64::<LittleEndian>(*last_sync_time)?;

                w.write_u64::<LittleEndian>(inserts.len() as u64)?;
                for (pid, ch) in inserts {
                    w.write_u8(1)?; // data_len
                    w.write_u8(*ch as u8)?;

                    w.write_u8(pid.0.len() as u8)?;
                    pid.write_bytes(&mut w);
                }

                w.write_u64::<LittleEndian>(deletes.len() as u64)?;
                for pid in deletes {
                    w.write_u8(pid.0.len() as u8)?;
                    pid.write_bytes(&mut w);
                }
            }

            SyncRequests::DocNameChange { document_id, name } => {
                w.write_u8(3)?;
                w.write_u128::<LittleEndian>(*document_id)?;
                w.write_all(name.to_string_lossy().as_bytes())?;
                w.write_all(b"\n")?;
            }

            SyncRequests::DeleteDoc { document_id } => {
                w.write_u8(4)?;
                w.write_u128::<LittleEndian>(*document_id)?;
            }
        }

        Ok(())
    }
    pub fn deserialize<R: Read>(reader: R) -> io::Result<Self> {
        let mut reader = io::BufReader::new(reader);

        let tag = reader.read_u8()?;

        Ok(match tag {
            0 => {
                let last_sync_time = reader.read_u64::<LittleEndian>()?;
                SyncRequests::SyncList { last_sync_time }
            }

            1 => {
                let document_id = reader.read_u128::<LittleEndian>()?;
                let last_sync_time = reader.read_u64::<LittleEndian>()?;

                SyncRequests::SyncDoc {
                    document_id,
                    last_sync_time,
                }
            }

            2 => {
                let document_id = reader.read_u128::<LittleEndian>()?;

                let mut name_buf = Vec::new();
                reader.read_until(b'\n', &mut name_buf)?;

                let name = if name_buf.len() > 1 {
                    name_buf.pop();
                    Some(PathBuf::from(String::from_utf8(name_buf)
                        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?))
                } else {
                    None
                };

                let last_sync_time = reader.read_u64::<LittleEndian>()?;

                let insert_count = reader.read_u64::<LittleEndian>()?;
                let mut inserts = Vec::with_capacity(insert_count as usize);

                for _ in 0..insert_count {
                    let data_len = reader.read_u8()?;
                    let mut data = vec![0u8; data_len as usize];
                    reader.read_exact(&mut data)?;

                    let ch = data[0] as char;

                    let depth = reader.read_u8()?;
                    let pid = Pid::read_bytes(&mut reader, depth as usize);

                    inserts.push((pid, ch));
                }

                let delete_count = reader.read_u64::<LittleEndian>()?;
                let mut deletes = Vec::with_capacity(delete_count as usize);

                for _ in 0..delete_count {
                    let depth = reader.read_u8()?;
                    let pid = Pid::read_bytes(&mut reader, depth as usize);
                    deletes.push(pid);
                }

                SyncRequests::SyncDocUpsert {
                    document_id,
                    name,
                    last_sync_time,
                    inserts,
                    deletes,
                }
            }

            3 => {
                let document_id = reader.read_u128::<LittleEndian>()?;

                let mut name_buf = Vec::new();
                reader.read_until(b'\n', &mut name_buf)?;
                name_buf.pop();

                let name = PathBuf::from(
                    String::from_utf8(name_buf)
                        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?,
                );

                SyncRequests::DocNameChange { document_id, name }
            }

            4 => {
                let document_id = reader.read_u128::<LittleEndian>()?;
                SyncRequests::DeleteDoc { document_id }
            }

            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unknown request type",
                ))
            }
        })
    }
}
#[derive(Debug)]
pub enum SyncResponses<'a> {
    SyncList(Vec<DocSyncInfo>),
    SyncDoc {
        document_id: u128,
        name: PathBuf,
        doc: &'a Doc,
    },
}

#[derive(Debug)]
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
                w.write_all(&[32u8])?;
                w.write_all(&(doc_sync_infos.len() as u64).to_le_bytes())?;
                for doc in doc_sync_infos {
                    w.write_all(&doc.last_mod_time.to_le_bytes())?;
                    w.write_all(&doc.document_id.to_le_bytes())?;
                }
            }
            SyncResponses::SyncDoc {
                document_id,
                name,
                doc,
            } => {
                w.write_all(&[33u8])?;
                w.write_all(&document_id.to_le_bytes())?;
                w.write_all(name.to_string_lossy().as_bytes());
                w.write_all(&[b'\n']);
                // Number of insert atoms:
                w.write_all(&(doc.char_len() as u64).to_le_bytes())?;
                doc.write_bytes(&mut w)?;
                // Number of delete atoms:
                w.write_all(&(0 as u64).to_le_bytes())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum DocOp {
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
