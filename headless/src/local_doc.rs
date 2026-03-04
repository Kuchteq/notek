use algos::{doc::{Doc, DocChar}, martree::Measured};

#[derive(Default)]
pub struct LocalDoc{
    pub crdt: Doc,
}

impl LocalDoc {
    pub fn insert_at_byte(&mut self, mut at: usize, text: String) {
        self.crdt.insert_text_at_bytepos(at, text);
    }
    pub fn delete_bytes(&mut self, start_byte: usize, len_byte: usize) {
        self.crdt.delete_byte_range(start_byte, len_byte);
    }
}
