use std::{collections::{BTreeMap, HashMap}, fs::{self, File}, io::{BufWriter, Write}, sync::mpsc};

use algos::{doc::Doc, structure::DocStructure};
use anyhow::{Result, anyhow};

use crate::editor_message::EditorMessage;

#[derive(Debug)]
pub struct State {
    pub docs: Vec<DocStructure>,
    pub current_doc: usize,
    pub by_time: BTreeMap<u64, usize>,
    pub by_id: HashMap<u128, usize>,
    pub by_name: HashMap<String, usize>,
}

impl State {
    pub fn init(dir: &str) -> Result<Self> {
        let mut s = State {
            docs: Vec::new(),
            current_doc: usize::MAX,
            by_time: BTreeMap::new(),
            by_id: HashMap::new(),
            by_name: HashMap::new(),
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
        self.by_name.insert(name, idx);
        self.docs.push(s);
        Ok(())
    }
    pub fn set_current_doc(&mut self, name: &String) {
        self.current_doc = *self.by_name.get(name).unwrap();
    }

    pub fn insert_in_current_doc(&mut self, pos: u32, text: &String) {
        let inserted = self.docs[self.current_doc].insert_text_at_bytepos(pos as usize, text);
    }
    pub fn delete_in_current_doc(&mut self, start: u32, len: u32) {
        let deleted = self.docs[self.current_doc].delete_byte_range(start as usize, len as usize);
    }

    pub fn flush_current_doc(&self) -> Result<()> {
        let current_doc = &self.docs[self.current_doc];
        current_doc.flush()?;
        Ok(())
    }

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
