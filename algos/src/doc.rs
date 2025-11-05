use byteorder::{LittleEndian, ReadBytesExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering, collections::BTreeMap, io::{Cursor, ErrorKind, Read, Write}
};
use anyhow::{anyhow, Context, Result};

use crate::{pid::{generate_between_pids, Pid}, pos::Pos, LBASE};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Doc {
    content: BTreeMap<Pid, char>,
}

impl Doc {
    pub fn new(content: &str) -> Doc {
        let beg = (Pid(vec![Pos { ident: 0, site: 0 }]), '_');
        let end = (
            Pid(vec![Pos {
                ident: LBASE,
                site: 0,
            }]),
            '_',
        );

        let mut d = Doc {
            content: BTreeMap::from([beg.clone(), end.clone()]),
        };
        if content.is_empty() {
            return d
        }
        let step = (u32::MAX as usize) / content.len();
        for (i, c) in content.chars().enumerate() {
            d.content.insert(
                Pid(vec![Pos {
                    ident: (i * step) as u32,
                    site: 1,
                }]),
                c,
            );
        }
        d
    }
    pub fn right(&self, pid: &Pid) -> Pid {
        self.content.range(pid..).skip(1).next().unwrap().0.clone()
    }
    pub fn left(&self, pid: &Pid) -> Pid {
        self.content.range(..pid).next_back().unwrap().0.clone()
    }

    pub fn offset(&self, pid: &Pid, offset: isize) -> Option<Pid> {
        if offset == 0 {
            return Some(pid.clone());
        }

        if offset > 0 {
            // move right
            self.content
                .range(pid..) // start at pid
                .skip(1) // skip self
                .nth((offset - 1) as usize)
                .map(|(k, _)| k.clone())
        } else {
            // move left
            self.content
                .range(..pid) // all before pid
                .rev() // backwards
                .nth((-offset - 1) as usize)
                .map(|(k, _)| k.clone())
        }
    }
    pub fn keys(&self) -> Vec<Pid> {
        self.content.keys().cloned().collect()
    }
    pub fn values(&self) -> Vec<char> {
        self.content.values().cloned().collect()
    }
    pub fn write_bytes_tobuf(&self, buf: &mut Vec<u8>) {
        for (pid, ch) in &self.content {
            // encode char (UTF-8, variable length)
            let mut cbuf = [0u8; 4];
            let encoded = ch.encode_utf8(&mut cbuf);
            // put atom's data length
            buf.push(encoded.len() as u8);
            // put atom's data
            buf.extend(encoded.as_bytes());
            // put pid's depth
            buf.push(pid.depth() as u8);
            // put pid vector
            pid.write_bytes(buf);
        }
    } 
    pub fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<()> {
        for (pid, ch) in &self.content {
            // Encode the char as UTF-8 (variable length)
            let mut cbuf = [0u8; 4];
            let encoded = ch.encode_utf8(&mut cbuf);

            // Write character length (1 byte)
            writer
                .write_all(&[encoded.len() as u8])
                .context("Failed to write character length")?;

            // Write UTF-8 bytes
            writer
                .write_all(encoded.as_bytes())
                .context("Failed to write character data")?;

            // Write pid depth (1 byte)
            writer
                .write_all(&[pid.depth() as u8])
                .context("Failed to write pid depth")?;

            // Write pid data (delegated to Pid)
            pid.write_bytes(writer)
                .context("Failed to write pid bytes")?;
        }

        writer.flush().context("Failed to flush writer")?;
        Ok(())
    }

    pub fn from_reader<R: Read>(reader: &mut R, n: usize) -> Self {
        let mut content = BTreeMap::new();

        for _ in 0..n {
            let data_len = reader.read_u8().unwrap() as usize;
            let mut bytes = [0u8; 4];
            reader.read_exact(&mut bytes[..data_len]).unwrap();
            let data = std::str::from_utf8(&bytes[..data_len])
                .unwrap()
                .chars()
                .next()
                .unwrap();
            let pid_depth = reader.read_u8().unwrap();
            let pid = Pid::from_reader(reader, pid_depth.into());
            content.insert(pid, data);
        }

        Doc { content }
    }
    pub fn from_reader_eof<R: Read>(reader: &mut R) -> Result<Self> {
        let mut content = BTreeMap::new();

        loop {
            let data_len = match reader.read_u8() {
                Ok(len) => len as usize,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e).context("Failed to read data length"),
            };

            let mut bytes = [0u8; 4];
            if let Err(e) = reader.read_exact(&mut bytes[..data_len]) {
                if e.kind() == ErrorKind::UnexpectedEof {
                    break;
                } else {
                    return Err(e).context("Failed to read data bytes");
                }
            }

            let data = std::str::from_utf8(&bytes[..data_len])
                .context("Invalid UTF-8 in data")?
                .chars()
                .next()
                .context("No character found in data")?;

            let pid_depth = match reader.read_u8() {
                Ok(depth) => depth,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e).context("Failed to read pid depth"),
            };

            let pid = Pid::from_reader(reader, pid_depth.into());
                

            content.insert(pid, data);
        }

        Ok(Doc { content })
    }

    pub fn insert(&mut self, pid: Pid, c: char) {
        self.content.insert(pid, c);
    }
    pub fn insert_left(&mut self, pid: Pid, c: char) -> Pid {
        let right = self.right(&pid);
        let new = generate_between_pids(&pid, &right, 1);
        self.content.insert(new.clone(), c);
        return new;
    }

    pub fn delete(&mut self, pid: &Pid) {
        self.content.remove(pid);
    }

    pub fn to_string(&self) -> String {
        self.content.values().collect()
    }
    pub fn len(&self) -> usize {
        return self.content.len();
    }
}
