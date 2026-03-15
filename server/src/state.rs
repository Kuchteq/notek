use std::{
    collections::{BTreeMap, HashMap}, env, fs, path::{Path, PathBuf}
};

use algos::{doc::Doc, structure::DocStructure, sync::{DocOp, DocSyncInfo, SyncResponses}};
use anyhow::Result;
use tokio::sync::{mpsc, oneshot};


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
                    if let Err(e) = r.serialize_into(&mut buf) {
                        eprintln!("Failed to serialize SyncDoc: {}", e);
                    }
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
                    if let Err(e) = r.serialize_into(&mut buf) {
                        eprintln!("Failed to serialize SyncList: {}", e);
                    }
                    let _ = respond_to.send(buf);
                }
                StateCommand::UpdateDoc { document_id, op } => {
                    let ds = &mut self.docs[self.by_id[&document_id]];
                    ds.applyOp(op);
                    println!("{:#?}", ds)
                }
                StateCommand::UpsertDoc { name, document_id } => {
                    if self.by_id.get(&document_id).is_none() {
                        if let Err(e) = self.add_doc(name, Some(document_id)) {
                            eprintln!("Failed to upsert doc {}: {}", document_id, e);
                        }
                    } else {
                        if let Err(e) = self.get_structure(document_id).update_name_after_external_rename(name.as_path()) {
                            eprintln!("Failed to rename doc {}: {}", document_id, e);
                        }
                    }
                }
                StateCommand::ChangeName { document_id, name } => {
                    if let Err(e) = self.get_structure(document_id).update_name_after_external_rename(name.as_path()) {
                        eprintln!("Failed to change name for doc {}: {}", document_id, e);
                    }
                }
                StateCommand::FlushChanges { document_id } => {
                    println!("flushed!");
                    if let Err(e) = self.get_structure(document_id).flush() {
                        eprintln!("Failed to flush doc {}: {}", document_id, e);
                    }
                }
                StateCommand::DeleteDoc { document_id } => {
                    if let Some(&idx) = self.by_id.get(&document_id) {
                        let removed_doc = self.docs.swap_remove(idx);
                        self.by_id.remove(&document_id);
                        self.by_time.remove(&removed_doc.last_modified);
                        if let Err(e) = removed_doc.delete_files() {
                            eprintln!("Failed to delete files for doc {}: {}", document_id, e);
                        }
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

