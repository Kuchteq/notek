use std::io::{self, Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub enum EditorMessage {
    Insert(u32, char),
    Delete(u32),
}

impl EditorMessage {
    pub fn deserialize<R: Read>(mut reader: R) -> io::Result<Self> {
        let v = reader.read_u32::<LittleEndian>()?;

        let op = (v >> 31) & 1;
        let index = v & 0x7FFF_FFFF;

        if op == 0 {
            let c = reader.read_u32::<LittleEndian>()?;
            let ch = char::from_u32(c)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid char"))?;
            Ok(EditorMessage::Insert(index, ch))
        } else {
            Ok(EditorMessage::Delete(index))
        }
    }

    pub fn deserialize_all<R: Read>(mut reader: R) -> io::Result<Vec<Self>> {
        let mut messages = Vec::new();

        loop {
            match Self::deserialize(&mut reader) {
                Ok(msg) => messages.push(msg),

                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    break;
                }

                Err(e) => return Err(e),
            }
        }

        Ok(messages)
    }
}
