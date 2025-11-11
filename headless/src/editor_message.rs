use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub enum EditorMessage {
    Update {
        inserts: Vec<(u32, char)>,
        deletes: Vec<u32>,
    },
}

impl EditorMessage {
    pub fn deserialize<R: Read>(mut reader: R) -> Self {
        // match reader.read_u8().unwrap() {
        //     _ => {
                let number_of_inserts = reader.read_u32::<LittleEndian>().unwrap();
                let inserts: Vec<(u32, char)> = (0..number_of_inserts)
                    .map(|_| {
                        let idx = reader.read_u32::<LittleEndian>().unwrap();
                        let char = reader.read_u32::<LittleEndian>().unwrap();
                        (idx, char::from_u32(char).unwrap())
                    })
                    .collect();

                let number_of_deletes = reader.read_u32::<LittleEndian>().unwrap();
                let deletes: Vec<u32> = (0..number_of_deletes)
                    .map(|_| {
                        reader.read_u32::<LittleEndian>().unwrap()
                    })
                    .collect();
                Self::Update { inserts: inserts, deletes: deletes }
            // }
        // }
    }
}
