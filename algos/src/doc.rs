use byteorder::{LittleEndian, ReadBytesExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering, collections::BTreeMap, io::{Cursor, Read}
};

use crate::{pid::{generate_between_pids, Pid}, pos::Pos, LBASE};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Doc {
    content: BTreeMap<Pid, char>,
}

impl Doc {
    pub fn new(content: String) -> Doc {
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
    pub fn write_bytes(&self, buf: &mut Vec<u8>) {
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
