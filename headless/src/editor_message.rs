use std::io::{self, Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub enum EditorMessage {
    Insert(u32, String),
    Delete(u32, u32),
}

impl EditorMessage {
    pub fn deserialize<R: Read>(mut reader: R) -> io::Result<Self> {
        let v = reader.read_u32::<LittleEndian>()?;

        let op = (v >> 31) & 1;
        let index = v & 0x7FFF_FFFF;
        let len = reader.read_u32::<LittleEndian>()?;
        if op == 0 {
            let mut buf = vec![0u8; len as usize];
            reader.read_exact(&mut buf)?;
            let text = String::from_utf8(buf).unwrap();
            Ok(EditorMessage::Insert(index, text))
        } else {
            Ok(EditorMessage::Delete(index, len))
        }
    }
}
