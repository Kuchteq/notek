use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::{self},
    path::{Path, PathBuf},
};

use algos::{doc::Doc, pid::Pid, structure::DocStructure};
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct State {
    pub docs: Vec<DocStructure>,
    pub current_doc: usize,
    pub base_dir: PathBuf,
    pub by_time: BTreeMap<u64, usize>,
    pub by_id: HashMap<u128, usize>,
    pub by_name: HashMap<PathBuf, usize>,
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
            current_doc: usize::MAX,
            base_dir: base_dir,
            by_time: BTreeMap::new(),
            by_id: HashMap::new(),
            by_name: HashMap::new(),
        };

        s.scan_dir_recursive(&s.base_dir.clone())?;
        Ok(s)
    }

    /// Recursively scan a directory for .md files and add them.
    fn scan_dir_recursive(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_dir_recursive(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let rel = path.strip_prefix(&self.base_dir).unwrap();
                self.add_doc(rel.to_path_buf(), None)?;
            }
        }
        Ok(())
    }

    pub fn add_doc(&mut self, name: PathBuf, upsertid: Option<u128>) -> Result<&DocStructure> {
        let mut s = DocStructure::load_or_create(&name, upsertid)?;
        let doc_id = s.id;
        let idx = self.docs.len();
        self.by_id.insert(s.id, idx);
        if self.by_time.contains_key(&s.last_modified) {
            s.last_modified += 1;
        }
        self.by_time.insert(s.last_modified, idx);
        self.by_name.insert(name, idx);
        println!("{}", s.get_doc().to_string());
        self.docs.push(s);
        Ok(self.docs.last().unwrap())
    }

    pub fn move_doc(&mut self, from: PathBuf, to: PathBuf) {
        if let Some(idx) = self.by_name.remove(&from) {
            if let Err(e) = self.docs[idx].update_name_after_external_rename(&to) {
                eprintln!("Failed to rename structure file for {:?}: {}", from, e);
            }
            self.by_name.insert(to, idx);
        }
    }

    pub fn set_current_doc(&mut self, name: &PathBuf) {
        println!("name: {:?}", name);
        if self.current_doc != usize::MAX {
            self.flush_current_doc();
        }
        let d = name.strip_prefix(&self.base_dir).unwrap();
        println!("{:?} {:?} {:?} {:?}", name, self.base_dir, self.by_name, d);
        self.current_doc = *self.by_name.get(d).unwrap();
    }

    pub fn insert_in_current_doc(&mut self, pos: u32, text: &String) -> Vec<(Pid, char)> {
        self.docs[self.current_doc].insert_text_at_bytepos(pos as usize, text)
    }
    pub fn delete_in_current_doc(&mut self, start: u32, len: u32) -> Vec<Pid> {
        self.docs[self.current_doc].delete_byte_range(start as usize, len as usize)
    }

    pub fn flush_current_doc(&self) -> Result<()> {
        let current_doc = &self.docs[self.current_doc];
        current_doc.flush()?;
        Ok(())
    }

    pub fn get_current_doc_id(&self) -> u128 {
        self.docs[self.current_doc].id
    }
    pub fn get_current_doc_name(&self) -> &PathBuf {
        &self.docs[self.current_doc].name
    }
    pub fn get_current_doc_crdt(&self) -> &Doc {
        &self.docs[self.current_doc].get_doc()
    }
    pub fn get_doc_by_name(&self, name: &PathBuf) -> &DocStructure {
        &self.docs[*self.by_name.get(name).unwrap()]
    }

    // pub fn get_doc_by_id(&self, id: ) -> &DocStructure {
    //     &self.docs[*self.by_name.get(name).unwrap()]
    // }

    // pub async fn run_state_manager(mut self, mut rx: mpsc::Receiver<EditorMessage>) {
    //     let mut current_doc = -1;
    //     while let Ok(cmd) = rx.recv() {
    //         match cmd {
    //             EditorMessage::Insert(pos, text) => todo!(),
    //             EditorMessage::Delete(start, len) => todo!(),
    //             EditorMessage::ChooseDocument(name) => {
    //                 self.by_name.get(&name);
    //             },
    //         }
    //     }
    // }
    // pub fn add_doc(&mut self, name: String, upsertid: Option<u128>) -> Result<()> {
    //     let mut s = DocStructure::load_or_create(&name, upsertid)?;
    //     let idx = self.docs.len();
    //     self.by_id.insert(s.id, idx);
    //     if self.by_time.contains_key(&s.last_modified) {
    //         s.last_modified += 1;
    //     }
    //     self.by_time.insert(s.last_modified, idx);
    //     self.docs.push(s);
    //     Ok(())
    // }

    // pub fn get_doc(&self, document_id: u128) -> &Doc {
    //     self.docs[self.by_id[&document_id]].get_doc()
    //     // self.docs.values().find(|d| d.id == document_id).unwrap().get_doc()
    // }
    //
    // pub fn get_structure(&mut self, document_id: u128) -> &mut DocStructure {
    //     &mut self.docs[self.by_id[&document_id]]
    //     // self.docs.values().find(|d| d.id == document_id).unwrap().get_doc()
    // }
}
