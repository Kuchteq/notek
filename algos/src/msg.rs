use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{doc::Doc, pid::Pid};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PeerMessage {
    Greet,
    Insert { site: u8, pid: Pid, c: char },
    Delete { site: u8, pid: Pid },
    NewSession { site: u8, doc: Doc },
}

impl PeerMessage {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            PeerMessage::Greet => vec![0u8],
            PeerMessage::NewSession { site, doc } => {
                // put header i.e. PeerMessage enum it is
                let mut buf = vec![1u8];
                // put site_id
                buf.push(*site);
                // put numberofatoms
                buf.extend((doc.len() as u64).to_le_bytes());
                doc.write_bytes_tobuf(&mut buf);
                buf
            }
            PeerMessage::Insert { site, pid, c } => {
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
            PeerMessage::Delete { site, pid } => {
                let mut buf = vec![3u8];
                buf.push(*site);
                buf.push(pid.depth() as u8);
                pid.write_bytes(&mut buf);
                buf
            }
        }
    }
    pub fn deserialize(buf: &[u8]) -> PeerMessage {
        let mut cur = Cursor::new(buf);
        match cur.read_u8().unwrap() {
            0u8 => PeerMessage::Greet,
            1u8 => {
                let site = cur.read_u8().unwrap();
                let number_of_atoms = cur.read_u64::<LittleEndian>().unwrap() as usize;
                PeerMessage::NewSession {
                    site: site,
                    doc: Doc::from_reader(&mut cur, number_of_atoms),
                }
            }
            2u8 => {
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
                PeerMessage::Insert { site: site, pid: pid, c: data }
            }
            3u8 => {
                let site = cur.read_u8().unwrap();
                let pid_depth = cur.read_u8().unwrap();
                let pid = Pid::from_reader(&mut cur, pid_depth as usize);
                PeerMessage::Delete { site: site, pid: pid }
            }
            _ => panic!(),
        }
    }
}
