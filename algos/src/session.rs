use std::{
    io::{self, BufRead, Cursor, Read},
    path::PathBuf,
};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{doc::Doc, pid::Pid};

#[derive(Debug, Clone)]
pub enum SessionMessage {
    Start {
        document_id: u128,
        last_sync_time: u64,
        name: Option<PathBuf>,
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
    ChangeName {
        name: PathBuf,
    },
}

impl SessionMessage {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            SessionMessage::Start {
                document_id,
                last_sync_time,
                name,
            } => {
                let mut buf = vec![64u8];
                buf.extend(last_sync_time.to_le_bytes());
                buf.extend(document_id.to_le_bytes());
                if let Some(name) = name {
                    buf.extend_from_slice(name.to_string_lossy().as_bytes());
                }
                buf.push(b'\n');
                buf
            }

            SessionMessage::Insert { site, pid, c } => {
                let mut buf = vec![65u8];

                // site
                buf.push(*site);

                // encode character
                let mut tmp = [0u8; 4];
                let encoded = c.encode_utf8(&mut tmp).as_bytes();

                // data length (must match deserialize read_u8)
                buf.push(encoded.len() as u8);

                // data bytes
                buf.extend_from_slice(encoded);

                // pid depth
                buf.push(pid.depth() as u8);

                // pid bytes
                pid.write_bytes(&mut buf);

                buf
            }

            SessionMessage::Delete { site, pid } => {
                let mut buf = vec![66u8];

                buf.push(*site);

                buf.push(pid.depth() as u8);

                pid.write_bytes(&mut buf);

                buf
            }

            SessionMessage::ChangeName { name } => {
                let mut buf = vec![67u8];

                // serialize name as UTF-8 bytes terminated by '\n'
                buf.extend_from_slice(name.to_string_lossy().as_bytes());
                buf.push(b'\n');

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
                let mut name_buf = Vec::new();
                cur.read_until(b'\n', &mut name_buf).unwrap();

                let name = (!name_buf.is_empty() && name_buf != b"\n").then(|| {
                    name_buf.pop();
                    PathBuf::from(String::from_utf8(name_buf).unwrap())
                });

                SessionMessage::Start {
                    document_id,
                    last_sync_time,
                    name,
                }
            }
            65u8 => {
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
                let pid = Pid::read_bytes(&mut cur, pid_depth as usize);
                SessionMessage::Insert {
                    site: site,
                    pid: pid,
                    c: data,
                }
            }
            66u8 => {
                let site = cur.read_u8().unwrap();
                let pid_depth = cur.read_u8().unwrap();
                let pid = Pid::read_bytes(&mut cur, pid_depth as usize);
                SessionMessage::Delete {
                    site: site,
                    pid: pid,
                }
            }
            67u8 => {
                let mut document_name = Vec::new();
                cur.read_until(b'\n', &mut document_name);
                SessionMessage::ChangeName {
                    name: PathBuf::from(String::from_utf8(document_name).unwrap()),
                }
            }
            _ => panic!(),
        }
    }
}
