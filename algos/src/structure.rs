use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use byteorder::{LittleEndian, ReadBytesExt};
use anyhow::Result;
use uuid::Uuid;

use crate::{doc::{Doc, DocChar}, pid::Pid, sync::DocOp};


#[derive(Debug)]
pub struct DocStructure {
    pub id: u128,
    pub last_modified: u64,
    pub name: PathBuf,
    pub state: DocState,
}

impl DocStructure {
    pub fn load_state(&mut self) -> Result<()> {
        let structure_path = self.get_structure_path();

        let file = File::open(structure_path)?;
        let mut reader = BufReader::new(file);
        reader.seek_relative(24)?;
        let doc = Doc::from_reader_eof(&mut reader)?;
        self.state = DocState::Cached(doc);
        Ok(())
    }

    pub fn get_doc(&self) -> &Doc {
        match self.state {
            DocState::Missing => {}
            DocState::Cached(_) => {}
        }

        if let DocState::Cached(doc) = &self.state {
            doc
        } else {
            unreachable!()
        }
    }

    fn apply_op(&mut self, op: DocOp) {
        match &mut self.state {
            DocState::Missing => {}
            DocState::Cached(doc) => match op {
                DocOp::Insert(pid, c) => doc.insert(pid, DocChar(c)),
                DocOp::Delete(pid) => doc.delete(&pid),
            },
        }
    }

    pub fn insert_text_at_bytepos(&mut self, pos: usize, text: &str) -> Vec<(Pid, char)> {
        match &mut self.state {
            DocState::Missing => todo!(),
            DocState::Cached(doc) => doc.insert_text_at_bytepos(pos, text),
        }
    }

    pub fn delete_byte_range(&mut self, start_byte: usize, len_byte: usize) -> Vec<Pid> {
        match &mut self.state {
            DocState::Missing => todo!(),
            DocState::Cached(doc) => doc.delete_byte_range(start_byte, len_byte),
        }
    }

    pub fn create_new(name: &Path, doc_id: u128) -> Result<Self> {
        let timestamp_ms =
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

        let plaintext_path = name.with_extension("md");

        let mut contents = String::new();
        if plaintext_path.exists() {
            contents = fs::read_to_string(&plaintext_path)?;
        }

        let ds = DocStructure {
            id: doc_id,
            name: name.to_path_buf(),
            last_modified: timestamp_ms,
            state: DocState::Cached(Doc::new(&contents)),
        };

        ds.flush()?;
        Ok(ds)
    }

    pub fn flush(&self) -> Result<()> {
        let structure_path = self.get_structure_path();
        let file = File::create(&structure_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&self.id.to_le_bytes())?;
        writer.write_all(&self.last_modified.to_le_bytes())?;

        if let DocState::Cached(doc) = &self.state {
            doc.write_bytes(&mut writer);
            writer.flush()?;

            let human_readable_path = self.get_plainmd_path();
            let human_readable = doc.to_string();
            fs::write(human_readable_path, human_readable)?;
        }

        Ok(())
    }

    fn delete_files(&self) -> Result<()> {
        fs::remove_file(self.get_structure_path())?;
        fs::remove_file(self.get_plainmd_path())?;
        Ok(())
    }

    fn get_structure_path(&self) -> PathBuf {
        self.name.with_extension("md.structure")
    }

    fn get_plainmd_path(&self) -> PathBuf {
        self.name.with_extension("md")
    }

    fn read_existing(structure_path: &Path, name: &Path) -> Result<Self> {
        let file = File::open(structure_path)?;
        let mut reader = BufReader::new(file);

        let id = reader.read_u128::<LittleEndian>()?;
        let last_modified = reader.read_u64::<LittleEndian>()?;
        let doc = Doc::from_reader_eof(&mut reader)?;

        Ok(DocStructure {
            id,
            name: name.to_path_buf(),
            last_modified,
            state: DocState::Cached(doc),
        })
    }

    pub fn load_or_create(name: &Path, upsertid: Option<u128>) -> Result<Self> {
        let structure_path = name.with_extension("md.structure");

        if structure_path.exists() {
            Self::read_existing(&structure_path, name)
        } else {
            let id = upsertid.unwrap_or(Uuid::new_v4().as_u128());
            Self::create_new(name, id)
        }
    }

    fn set_name(&mut self, name: &Path) -> Result<()> {
        println!("old {:?} new {:?}", self.name, name);

        fs::rename(self.get_structure_path(), name.with_extension("md.structure"))?;
        fs::rename(self.get_plainmd_path(), name.with_extension("md"))?;

        self.name = name.to_path_buf();
        Ok(())
    }
}

#[derive(Debug)]
enum DocState {
    Missing,
    Cached(Doc),
}
