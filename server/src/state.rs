use std::{
    collections::{BTreeMap, HashMap}, fs::{self, File}, io::{BufReader, BufWriter, Write}, path::Path, rc::Rc, time::{SystemTime, UNIX_EPOCH}
};

use algos::doc::Doc;
use anyhow::{Result, anyhow};
use byteorder::{LittleEndian, ReadBytesExt};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::sync::{DocSyncInfo, SyncResponses};

#[derive(Debug)]
pub struct State {
    pub docs: BTreeMap<u64, Rc<DocStructure>>,
    pub by_id: HashMap<u128, Rc<DocStructure>>,
}

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
        doc: DocStructure,
    },
}

impl State {
    pub fn init(dir: &str) -> Result<Self> {
        let mut dt = BTreeMap::new();
        let mut di = HashMap::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let name = path
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid UTF-8 path: {:?}", path))?
                    .to_string();

                let structure = DocStructure::init_from_filepath(name)?;
                let s = Rc::new(structure);
                dt.insert(s.last_modified, s.clone());
                di.insert(s.id, s);
            }
        }
        Ok(State { docs: dt, by_id: di })
    }

    pub fn get_doc(&self, document_id: u128) -> &Doc {
        self.docs.values().find(|d| d.id == document_id).unwrap().get_doc()
    }

    pub async fn run_state_manager(mut self, mut rx: mpsc::Receiver<StateCommand>) {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                StateCommand::GetSyncFullDoc {
                    document_id,
                    respond_to,
                } => {
                    let doc = self.get_doc(document_id);
                    let r = SyncResponses::SyncFullDoc { document_id: document_id, doc: doc };
                    let mut buf = Vec::new();
                    r.serialize_into(&mut buf);
                    let _ = respond_to.send(buf);
                }
                StateCommand::GetSyncList { last_sync_time, respond_to } => {
                    let docs = self
                        .docs
                        .iter()
                        .map(|(k, v)| DocSyncInfo::new(*k, v.id))
                        .collect();
                    let r = SyncResponses::SyncList(docs);
                    let mut buf = Vec::new();
                    r.serialize_into(&mut buf);
                    let _ = respond_to.send(buf);
                }
                StateCommand::UpdateDoc { document_id, doc } => {
                    let d = self.by_id.get(&document_id).unwrap();
                    // d.name = String::new();
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
    fn init_from_filepath(name: String) -> Result<DocStructure> {
        let structure_path = &format!("{}.structure", name);
        let structure_path = Path::new(structure_path);

        if structure_path.exists() {
            let file = File::open(structure_path)?;
            let mut reader = BufReader::new(file);
            let id = reader.read_u128::<LittleEndian>()?;
            let last_modified = reader.read_u64::<LittleEndian>()?;
            let doc = Doc::from_reader_eof(&mut reader)?;

            return Ok(DocStructure {
                id,
                name,
                last_modified,
                state: DocState::Cached(doc),
            });
        } else {
            let f = File::create(structure_path)?;
            let mut writer = BufWriter::new(f);

            let id: u128 = Uuid::new_v4().as_u128();
            let timestamp_ms: u64 =
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

            writer.write_all(&id.to_le_bytes())?;
            writer.write_all(&timestamp_ms.to_le_bytes())?;

            let doc = Doc::new(fs::read_to_string(&name)?);
            doc.write_bytes(&mut writer)?;
            writer.flush()?;

            return Ok(DocStructure {
                id,
                name,
                last_modified: timestamp_ms,
                state: DocState::Cached(doc),
            });
        }
    }
}

#[derive(Debug)]
enum DocState {
    Missing,
    Cached(Doc),
}
