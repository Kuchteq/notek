use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use uuid::Uuid;

use crate::{
    doc::{Doc, DocChar},
    pid::Pid,
    sync::DocOp,
};

/// Given a base name like `school/math/note.md`, returns `school/math/.note.md.structure`.
fn hidden_structure_path(name: &Path) -> PathBuf {
    let parent = name.parent().unwrap_or(Path::new(""));
    let stem = name.file_stem().unwrap_or_default();
    let hidden_name = format!(".{}.md.structure", stem.to_string_lossy());
    parent.join(hidden_name)
}

/// Given a base name like `school/math/note.md`, returns `school/math/.note.md.oplog`.
fn hidden_oplog_path(name: &Path) -> PathBuf {
    let parent = name.parent().unwrap_or(Path::new(""));
    let stem = name.file_stem().unwrap_or_default();
    let hidden_name = format!(".{}.md.oplog", stem.to_string_lossy());
    parent.join(hidden_name)
}

#[derive(Debug)]
pub struct DocStructure {
    pub id: u128,
    pub last_modified: u64,
    pub name: PathBuf,
    pub state: DocState,
    pub oplog: Vec<DocOp>,
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
            DocState::Cached(doc) => {
                let inserted = doc.insert_text_at_bytepos(pos, text);
                for (pid, c) in &inserted {
                    self.oplog.push(DocOp::Insert(pid.clone(), *c));
                }
                inserted
            }
        }
    }

    pub fn delete_byte_range(&mut self, start_byte: usize, len_byte: usize) -> Vec<Pid> {
        match &mut self.state {
            DocState::Missing => todo!(),
            DocState::Cached(doc) => {
                let deleted = doc.delete_byte_range(start_byte, len_byte);
                for pid in &deleted {
                    self.oplog.push(DocOp::Delete(pid.clone()));
                }
                deleted
            }
        }
    }

    pub fn applyOp(&mut self, op: DocOp) {
        match &mut self.state {
            DocState::Missing => todo!(),
            DocState::Cached(doc) => match op {
                DocOp::Insert(pid, c) => doc.insert(pid, DocChar(c)),
                DocOp::Delete(pid) => doc.delete(&pid),
            },
        }
    }
    pub fn create_new(name: &Path, doc_id: u128) -> Result<Self> {
        let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

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
            oplog: Vec::new(),
        };

        if let Some(parent) = name.parent() {
            fs::create_dir_all(parent)?;
        }

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
            // let human_readable_path = self.get_plainmd_path();
            // let human_readable = doc.to_string();
            // fs::write(human_readable_path, human_readable)?;
        }

        Ok(())
    }

    /// Append current in-memory oplog to the on-disk oplog file, then clear memory.
    pub fn flush_oplog(&mut self) -> Result<()> {
        if self.oplog.is_empty() {
            return Ok(());
        }
        let path = self.get_oplog_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        for op in &self.oplog {
            op.write_to(&mut writer)?;
        }
        writer.flush()?;
        self.oplog.clear();
        Ok(())
    }

    /// Load oplog from disk. Returns empty vec if file doesn't exist or is unreadable.
    fn load_oplog(name: &Path) -> Vec<DocOp> {
        let path = hidden_oplog_path(name);
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };
        let mut reader = BufReader::new(file);
        let mut ops = Vec::new();
        loop {
            match DocOp::read_from(&mut reader) {
                Ok(op) => ops.push(op),
                Err(_) => break,
            }
        }
        ops
    }

    /// Drain all ops from the oplog, returning them and clearing both memory and disk.
    pub fn drain_oplog(&mut self) -> Result<Vec<DocOp>> {
        // First flush any in-memory ops to disk so we capture everything
        self.flush_oplog()?;
        // Reload from disk to get the full set
        let ops = Self::load_oplog(&self.name);
        // Clear the on-disk file
        self.clear_oplog_file()?;
        Ok(ops)
    }

    /// Remove the oplog file from disk.
    fn clear_oplog_file(&self) -> Result<()> {
        let path = self.get_oplog_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn get_oplog_path(&self) -> PathBuf {
        hidden_oplog_path(&self.name)
    }

    pub fn delete_files(&self) -> Result<()> {
        fs::remove_file(self.get_structure_path())?;
        fs::remove_file(self.get_plainmd_path())?;
        let oplog_path = self.get_oplog_path();
        if oplog_path.exists() {
            fs::remove_file(&oplog_path)?;
        }
        Ok(())
    }

    fn get_structure_path(&self) -> PathBuf {
        hidden_structure_path(&self.name)
    }

    fn get_plainmd_path(&self) -> PathBuf {
        self.name.with_extension("md")
    }

    pub fn read_existing(structure_path: &Path, name: &Path) -> Result<Self> {
        let file = File::open(structure_path)?;
        let mut reader = BufReader::new(file);

        let id = reader.read_u128::<LittleEndian>()?;
        let last_modified = reader.read_u64::<LittleEndian>()?;
        let doc = Doc::from_reader_eof(&mut reader)?;

        let oplog = Self::load_oplog(name);

        Ok(DocStructure {
            id,
            name: name.to_path_buf(),
            last_modified,
            state: DocState::Cached(doc),
            oplog,
        })
    }

    pub fn load_or_create(name: &Path, upsertid: Option<u128>) -> Result<Self> {
        let structure_path = hidden_structure_path(name);

        if structure_path.exists() {
            Self::read_existing(&structure_path, name)
        } else {
            let id = upsertid.unwrap_or(Uuid::new_v4().as_u128());
            Self::create_new(name, id)
        }
    }

    pub fn set_name(&mut self, name: &Path) -> Result<()> {
        println!("old {:?} new {:?}", self.name, name);

        fs::rename(self.get_structure_path(), hidden_structure_path(name))?;
        fs::rename(self.get_plainmd_path(), name.with_extension("md"))?;

        let old_oplog = self.get_oplog_path();
        let new_oplog = hidden_oplog_path(name);
        if old_oplog.exists() {
            fs::rename(&old_oplog, &new_oplog)?;
        }

        self.name = name.to_path_buf();
        Ok(())
    }

    /// Update the name after an external rename of the .md file.
    /// Only renames the hidden .md.structure companion file (the .md was already
    /// renamed by the OS / another process).
    pub fn update_name_after_external_rename(&mut self, new_name: &Path) -> Result<()> {
        let old_structure = self.get_structure_path();
        let new_structure = hidden_structure_path(new_name);

        // Ensure parent directories exist (e.g. moving note.md -> school/math/note.md)
        if let Some(parent) = new_structure.parent() {
            fs::create_dir_all(parent)?;
        }

        if old_structure.exists() {
            fs::rename(&old_structure, &new_structure)?;
        }

        let old_oplog = self.get_oplog_path();
        let new_oplog = hidden_oplog_path(new_name);
        if old_oplog.exists() {
            fs::rename(&old_oplog, &new_oplog)?;
        }

        self.name = new_name.to_path_buf();
        Ok(())
    }
}

#[derive(Debug)]
enum DocState {
    Missing,
    Cached(Doc),
}
