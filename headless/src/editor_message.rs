use std::io::{self, Read};

#[derive(Debug)]
pub enum EditorMessage {
    Insert(u32, String),
    Delete(u32, u32),
    ChooseDocument(String),
    Flush,
}

impl EditorMessage {
    pub fn deserialize<R: Read>(mut reader: R) -> io::Result<Self> {
        use byteorder::{LittleEndian, ReadBytesExt};

        let opcode = match reader.read_u8() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        match opcode {
            // Insert
            0 => {
                let index = reader.read_u32::<LittleEndian>()?;
                let len = reader.read_u32::<LittleEndian>()?;

                let mut buf = vec![0u8; len as usize];
                reader.read_exact(&mut buf)?;

                let text = String::from_utf8(buf)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

                Ok(EditorMessage::Insert(index, text))
            }

            // Delete
            1 => {
                let index = reader.read_u32::<LittleEndian>()?;
                let len = reader.read_u32::<LittleEndian>()?;

                Ok(EditorMessage::Delete(index, len))
            }

            // Choose document (read until EOF)
            2 => {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;

                let name = String::from_utf8(buf)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

                Ok(EditorMessage::ChooseDocument(name))
            }

            3 => {
                Ok(EditorMessage::Flush)
            }

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown opcode",
            )),
        }
    }
}
