use anyhow::{Context, Result, anyhow};
use byteorder::{LittleEndian, ReadBytesExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    io::{Cursor, ErrorKind, Read, Write},
};

use crate::{
    LBASE,
    martree::{MarTree, Measured},
    pid::{Pid, generate_between_pids},
    pos::Pos,
};

/// A wrapper around char that measures its UTF-8 byte length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocChar(pub char);

#[derive(Debug, Clone)]
pub struct Doc {
    content: MarTree<Pid, DocChar>,
}

impl Default for Doc {
    fn default() -> Self {
        Doc::new("")
    }
}

impl Measured for DocChar {
    fn measured(&self) -> usize {
        self.0.len_utf8()
    }
}

impl Doc {
    pub fn new(content: &str) -> Doc {
        let beg = (Pid(vec![Pos { ident: 0, site: 0 }]), DocChar('_'));
        let end = (
            Pid(vec![Pos {
                ident: LBASE,
                site: 0,
            }]),
            DocChar('_'),
        );

        let mut d = Doc {
            content: MarTree::from_iter([beg.clone(), end.clone()]),
        };
        if content.is_empty() {
            return d;
        }
        let step = (u32::MAX as usize) / content.len();
        for (i, c) in content.chars().enumerate() {
            d.content.insert(
                Pid(vec![Pos {
                    ident: (i * step) as u32,
                    site: 1,
                }]),
                DocChar(c),
            );
        }
        d
    }
    pub fn offset(&self, pid: &Pid, offset: isize) -> Option<Pid> {
        return None;
        // if offset == 0 {
        //     return Some(pid.clone());
        // }
        //
        // if offset > 0 {
        //     // move right
        //     self.content
        //         .range(pid..) // start at pid
        //         .skip(1) // skip self
        //         .nth((offset - 1) as usize)
        //         .map(|(k, _)| k.clone())
        // } else {
        //     // move left
        //     self.content
        //         .range(..pid) // all before pid
        //         .rev() // backwards
        //         .nth((-offset - 1) as usize)
        //         .map(|(k, _)| k.clone())
        // }
    }
    // pub fn keys(&self) -> Vec<Pid> {
    //     self.content.keys().cloned().collect()
    // }
    // pub fn values(&self) -> Vec<char> {
    //     self.content.values().cloned().collect()
    // }
    pub fn write_bytes_tobuf(&self, buf: &mut Vec<u8>) {
        for (pid, ch) in self.content.iter() {
            // encode char (UTF-8, variable length)
            let mut cbuf = [0u8; 4];
            let encoded = ch.0.encode_utf8(&mut cbuf);
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
        for (pid, ch) in self.content.iter() {
            // Encode the char as UTF-8 (variable length)
            let mut cbuf = [0u8; 4];
            let encoded = ch.0.encode_utf8(&mut cbuf);

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
        let mut content = MarTree::default();

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
            content.insert(pid, DocChar(data));
        }

        Doc { content }
    }
    pub fn from_reader_eof<R: Read>(reader: &mut R) -> Result<Self> {
        let mut content = MarTree::default();

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

            content.insert(pid, DocChar(data));
        }

        Ok(Doc { content })
    }

    pub fn insert(&mut self, pid: Pid, c: DocChar) {
        self.content.insert(pid, c);
    }
    pub fn insert_leftof(&mut self, pid: &Pid, c: DocChar) -> Pid {
        let right = &self.content.get_next(pid).unwrap().0;
        let new = generate_between_pids(pid, &right, 1);
        self.content.insert(new.clone(), c);
        return new;
    }
    pub fn insert_at_idx(&mut self, idx: usize, c: DocChar) -> Pid {
        let left = &self.content.get_by_index(idx).unwrap().0;
        let right = &self.content.get_next(left).unwrap().0;
        let new = generate_between_pids(left, right, 1);
        self.content.insert(new.clone(), c);
        return new;
    }
    pub fn insert_at_bytepos(&mut self, pos: usize, c: DocChar) -> Pid {
        let left = &self.content.get_by_alt_size(pos).unwrap().0;
        let right = &self.content.get_next(left).unwrap().0;
        let new = generate_between_pids(left, right, 1);
        self.content.insert(new.clone(), c);
        return new;
    }

    pub fn insert_text_at_bytepos(&mut self, pos: usize, text: String) -> Vec<(Pid, char)> {
        let mut pos = self.content.alt_to_index(pos);
        // ik its in bytes but it's a good enough heuristic
        let mut inserted = Vec::with_capacity(text.len());
        for c in text.chars() {
            inserted.push((self.insert_at_idx(pos, DocChar(c)), c));
            pos += 1;
        }
        inserted
    }

    pub fn delete(&mut self, pid: &Pid) {
        self.content.remove(pid);
    }

    pub fn delete_at_idx(&mut self, idx: usize) -> Pid {
        // TODO Avoid this clone by writing the function from scratch
        let key = self.content.get_by_index(idx + 1).unwrap().0.clone();
        self.content.remove(&key);
        key
    }
    pub fn delete_byte_range(&mut self, start_byte: usize, len_byte: usize) -> Vec<Pid> {
        let start_idx = self.content.alt_to_index(start_byte);
        let end_idx = self.content.alt_to_index(start_byte+len_byte);
        let mut deleted = Vec::with_capacity(end_idx-start_idx);
        for i in (start_idx..end_idx).rev() {
            deleted.push(self.delete_at_idx(i));
        }
        deleted
    }

    pub fn to_string(&self) -> String {
        // self.content.iter().map(|(_,v)| v.0).collect()[1..self.char_len() - 1]
        self.content
            .iter()
            .skip(1)
            .take(self.char_len().saturating_sub(2))
            .map(|(_, v)| v.0)
            .collect::<String>()
    }
    pub fn to_abs_string(&self) -> String {
        // self.content.iter().map(|(_,v)| v.0).collect()[1..self.char_len() - 1]
        self.content
            .iter()
            .map(|(_, v)| v.0)
            .collect::<String>()
    }
    pub fn char_len(&self) -> usize {
        return self.content.size();
    }
    pub fn byte_len(&self) -> usize {
        return self.content.size_alt();
    }
}
