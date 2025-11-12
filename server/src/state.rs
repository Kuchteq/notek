use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use algos::{doc::Doc, sync::{DocOp, DocSyncInfo, SyncResponses}};
use anyhow::{Result, anyhow};
use byteorder::{LittleEndian, ReadBytesExt};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;


#[derive(Debug)]
pub struct State {
    pub docs: Vec<DocStructure>,
    pub by_time: BTreeMap<u64, usize>,
    pub by_id: HashMap<u128, usize>,
}

#[derive(Debug)]
pub enum StateCommand {
    GetSyncFullDoc {
        document_id: u128,
        // The state manager responds already with a serialized buffer
        respond_to: oneshot::Sender<Vec<u8>>,
    },
    GetSyncList {
        last_sync_time: u64,
        // The state manager responds already with a serialized buffer
        respond_to: oneshot::Sender<Vec<u8>>,
    },
    UpdateDoc {
        document_id: u128,
        op: DocOp,
    },
    UpsertDoc {
        document_id: u128,
    },
    DeleteDoc {
        document_id: u128,
    },
    ChangeName {
        document_id: u128,
        name: String,
    },
    FlushChanges {
        document_id: u128,
    },
}

impl State {
    pub fn init(dir: &str) -> Result<Self> {
        let mut s = State {
            docs: Vec::new(),
            by_time: BTreeMap::new(),
            by_id: HashMap::new(),
        };

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let name = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid UTF-8 path: {:?}", path))?
                    .to_string();

                s.add_doc(name, None)?;
            }
        }
        Ok(s)
    }
    pub fn add_doc(&mut self, name: String, upsertid: Option<u128>) -> Result<()> {
        let mut s = DocStructure::load_or_create(&name, upsertid)?;
        let idx = self.docs.len();
        self.by_id.insert(s.id, idx);
        if self.by_time.contains_key(&s.last_modified) {
            s.last_modified += 1;
        }
        self.by_time.insert(s.last_modified, idx);
        self.docs.push(s);
        Ok(())
    }

    pub fn get_doc(&self, document_id: u128) -> &Doc {
        self.docs[self.by_id[&document_id]].get_doc()
        // self.docs.values().find(|d| d.id == document_id).unwrap().get_doc()
    }

    pub fn get_structure(&mut self, document_id: u128) -> &mut DocStructure {
        &mut self.docs[self.by_id[&document_id]]
        // self.docs.values().find(|d| d.id == document_id).unwrap().get_doc()
    }

    pub async fn run_state_manager(mut self, mut rx: mpsc::Receiver<StateCommand>) {
        while let Some(cmd) = rx.recv().await {
            println!("the cmd {:#?}", cmd);
            match cmd {
                StateCommand::GetSyncFullDoc {
                    document_id,
                    respond_to,
                } => {
                    let structure = self.get_structure(document_id);
                    let r = SyncResponses::SyncDoc {
                        document_id: document_id,
                        name: structure.name.clone(),
                        doc: structure.get_doc(),
                    };
                    let mut buf = Vec::new();
                    r.serialize_into(&mut buf);
                    let _ = respond_to.send(buf);
                }
                StateCommand::GetSyncList {
                    last_sync_time,
                    respond_to,
                } => {
                    let docs = self
                        .by_time
                        .iter()
                        // .filter(|&(&t, _)| t >= last_sync_time)
                        .map(|(&t, &i)| DocSyncInfo::new(t, self.docs[i].id))
                        .collect();
                    let r = SyncResponses::SyncList(docs);
                    println!("the synclist {:#?}", r);
                    let mut buf = Vec::new();
                    r.serialize_into(&mut buf);
                    let _ = respond_to.send(buf);
                }
                StateCommand::UpdateDoc { document_id, op } => {
                    let ds = &mut self.docs[self.by_id[&document_id]];
                    ds.applyOp(op);
                    println!("{:#?}", ds)
                }
                StateCommand::UpsertDoc { document_id } => {
                    if let None = self.by_id.get(&document_id) {
                        let name = format!("{}-unnamed", document_id);
                        let filename = format!("{}.md", name);
                        OpenOptions::new().create(true).write(true).open(&filename);
                        self.add_doc(name, Some(document_id));
                    }
                }
                StateCommand::ChangeName { document_id, name } => {
                    self.get_structure(document_id).set_name(&name);
                }
                StateCommand::FlushChanges { document_id } => {
                    println!("flushed!");
                    self.get_structure(document_id).flush();
                }
                StateCommand::DeleteDoc { document_id } => {
                    if let Some(&idx) = self.by_id.get(&document_id) {
                        let removed_doc = self.docs.swap_remove(idx);
                        self.by_id.remove(&document_id);
                        self.by_time.remove(&removed_doc.last_modified);
                        removed_doc.delete_files();
                        if idx < self.docs.len() {
                            let moved_doc = &self.docs[idx];
                            self.by_id.insert(moved_doc.id, idx);
                            self.by_time.insert(moved_doc.last_modified, idx);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct DocStructure {
    pub id: u128,
    last_modified: u64, // todo, change this, this field shouldn't be duplicating the key of the
    // btree
    name: String,
    pub state: DocState,
}

impl DocStructure {
    fn load_state(&mut self) -> Result<()> {
        let structure_path = &format!("{}.structure", self.name);
        let structure_path = Path::new(structure_path);

        let file = File::open(structure_path)?;
        let mut reader = BufReader::new(file);
        reader.seek_relative(24)?;
        let doc = Doc::from_reader_eof(&mut reader)?;
        self.state = DocState::Cached(doc);
        Ok(())
    }
    fn get_doc(&self) -> &Doc {
        match self.state {
            DocState::Missing => {} // TODO self.load_state().unwrap(),
            DocState::Cached(_) => {}
        }

        if let DocState::Cached(doc) = &self.state {
            doc
        } else {
            unreachable!()
        }
    }
    fn applyOp(&mut self, op: DocOp) {
        match &mut self.state {
            DocState::Missing => {} // TODO self.load_state().unwrap(),
            DocState::Cached(doc) => match op {
                DocOp::Insert(pid, c) => doc.insert(pid, c),
                DocOp::Delete(pid) => doc.delete(&pid),
            },
        }
    }
    // fn create_structure_for_existing(name)
    fn create_new(name: &str, doc_id: u128) -> Result<Self> {
        let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

        let plaintext = format!("{}.md", name);
        let plaintext_path = Path::new(&plaintext);
        let mut contents = String::new();
        if plaintext_path.exists() {
            contents = fs::read_to_string(plaintext_path).unwrap();
        }

        let ds = DocStructure {
            id: doc_id,
            name: name.to_string(),
            last_modified: timestamp_ms,
            state: DocState::Cached(Doc::new(&contents)),
        };
        ds.flush();

        Ok(ds)
    }
    fn flush(&self) -> Result<()> {
        let structure_path_str = format!("{}.md.structure", self.name);
        let file = File::create(structure_path_str)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&self.id.to_le_bytes())?;
        writer.write_all(&self.last_modified.to_le_bytes())?;
        if let DocState::Cached(doc) = &self.state {
            doc.write_bytes(&mut writer);
            let human_readable_path = format!("{}.md", self.name);
            let human_readable = &String::from_iter(doc.values().iter())[1..doc.len() - 1];
            std::fs::write(human_readable_path, human_readable);
        }
        writer.flush();

        Ok(())
    }

    fn delete_files(&self) -> Result<()> {
        fs::remove_file(self.get_structure_path());
        fs::remove_file(self.get_plainmd_path());
        Ok(())
    }

    fn get_structure_path(&self) -> String {
        format!("{}.md.structure",self.name)
    }
    fn get_plainmd_path(&self) -> String {
        format!("{}.md",self.name)
    }

    fn read_existing(structure_path: &Path, name: String) -> Result<Self> {
        let file = File::open(structure_path)?;
        let mut reader = BufReader::new(file);

        let id = reader.read_u128::<LittleEndian>()?;
        let last_modified = reader.read_u64::<LittleEndian>()?;
        let doc = Doc::from_reader_eof(&mut reader)?;

        Ok(DocStructure {
            id,
            name,
            last_modified,
            state: DocState::Cached(doc),
        })
    }

    fn load_or_create(name: &str, upsertid: Option<u128>) -> Result<Self> {
        let structure_path_str = format!("{}.md.structure", name);
        let structure_path = Path::new(&structure_path_str);

        if structure_path.exists() {
            Self::read_existing(structure_path, name.to_string())
        } else {
            let id = upsertid.unwrap_or(Uuid::new_v4().as_u128());
            Self::create_new(name, id)
        }
    }

    fn set_name(&mut self, name: &str) -> anyhow::Result<()> {
        println!("old {} new {}", self.name, name);
        fs::rename(
            format!("{}.md.structure", self.name),
            format!("{}.md.structure", name),
        )?;
        fs::rename(format!("{}.md", self.name), format!("{}.md", name))?;
        self.name = name.to_string();
        Ok(())
    }
}

#[derive(Debug)]
enum DocState {
    Missing,
    Cached(Doc),
}
