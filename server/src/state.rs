use std::{
    collections::{BTreeMap, HashMap}, env, fs::{self, OpenOptions}, path::{Path, PathBuf}
};

use algos::{doc::{Doc, DocChar}, structure::DocStructure, sync::{DocOp, DocSyncInfo, SyncResponses}};
use anyhow::{Result, anyhow};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;


#[derive(Debug)]
pub struct State {
    pub docs: Vec<DocStructure>,
    pub base_dir: PathBuf,
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
        name: PathBuf
    },
    DeleteDoc {
        document_id: u128,
    },
    ChangeName {
        document_id: u128,
        name: PathBuf,
    },
    FlushChanges {
        document_id: u128,
    },
}

impl State {
    pub fn init(dir: &Path) -> Result<Self> {
        let base_dir = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            env::current_dir()?.join(dir)
        };

        let base_dir = std::fs::canonicalize(base_dir)?;

        let mut s = State {
            docs: Vec::new(),
            base_dir: base_dir,
            by_time: BTreeMap::new(),
            by_id: HashMap::new(),
        };

        for entry in fs::read_dir(&s.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            let path = path.strip_prefix(&s.base_dir).unwrap();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                s.add_doc(path.to_path_buf(), None)?;
            }
        }
        Ok(s)
    }

    pub fn add_doc(&mut self, name: PathBuf, upsertid: Option<u128>) -> Result<()> {
        let mut s = DocStructure::load_or_create(&name, upsertid)?;
        let idx = self.docs.len();
        self.by_id.insert(s.id, idx);
        if self.by_time.contains_key(&s.last_modified) {
            s.last_modified += 1;
        }
        self.by_time.insert(s.last_modified, idx);
        println!("{}", s.get_doc().to_string());
        self.docs.push(s);
        Ok(())
    }

    // pub fn move_doc(&mut self, from: PathBuf, to: PathBuf) {
    //     if let Some(idx) = self.by_name.remove(&from) {
    //         if let Err(e) = self.docs[idx].update_name_after_external_rename(&to) {
    //             eprintln!("Failed to rename structure file for {:?}: {}", from, e);
    //         }
    //         self.by_name.insert(to, idx);
    //     }
    // }
    pub fn get_doc(&self, document_id: u128) -> &Doc {
        self.docs[self.by_id[&document_id]].get_doc()
        // self.docs.values().find(|d| d.id == document_id).unwrap().get_doc()
    }

    pub fn get_structure(&mut self, document_id: u128) -> &mut DocStructure {
        &mut self.docs[*self.by_id.get(&document_id).unwrap()]
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
                StateCommand::UpsertDoc { name, document_id } => {
                    if let None = self.by_id.get(&document_id) {
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
enum DocState {
    Missing,
    Cached(Doc),
}
