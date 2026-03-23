use std::{
    collections::{BTreeMap, VecDeque},
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    time::Duration,
    u128,
};

use algos::{session::SessionMessage, sync::DocOp};
use anyhow::Result;
use tungstenite::{Message, WebSocket, connect, stream::MaybeTlsStream};

pub struct Oplog {
    pub current_document: u128,
    pub log: BTreeMap<u128, VecDeque<DocOp>>,
}

/// Given a base name like `school/math/note.md`, returns `school/math/.note.md.oplog`.
fn hidden_oplog_path(name: &Path) -> PathBuf {
    let parent = name.parent().unwrap_or(Path::new(""));
    let stem = name.file_stem().unwrap_or_default();
    let hidden_name = format!(".{}.md.oplog", stem.to_string_lossy());
    parent.join(hidden_name)
}

impl Oplog {
    pub fn init() -> Result<Self> {
        Ok(Oplog {
            current_document: u128::MAX,
            log: BTreeMap::new(),
        })
    }

    pub fn run(&mut self, rx: Receiver<SessionMessage>) {
        let (mut ws, _) = connect("ws://127.0.0.1:9001").unwrap();
        // let msg = Message::from(cmd.serialize());
        // let _ = ws.send(msg);

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(event) => match event {
                    SessionMessage::Insert { pid, c, .. } => {
                        let log = self.log.get_mut(&self.current_document).unwrap();
                        log.push_back(DocOp::Insert(pid, c));
                    }
                    SessionMessage::Start { document_id, .. } => {
                        self.current_document = document_id;
                        self.log.insert(document_id, VecDeque::new());
                        ws.send(Message::from(event.serialize()));
                    }
                    SessionMessage::Delete { site, pid } => {
                        let log = self.log.get_mut(&self.current_document).unwrap();
                        log.push_back(DocOp::Delete(pid));
                    }
                    SessionMessage::ChangeName { name } => todo!(),
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // no message → fall through to sending
                }
                Err(_) => break,
            }

            self.flush_to_server(&mut ws);
        }
    }

    pub fn flush_to_server(&mut self, ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
        if let Some(queue) = self.log.get_mut(&self.current_document) {
            if let Some(op) = queue.pop_front() {
                let msg = match op {
                    DocOp::Insert(pid, c) => SessionMessage::Insert { site: 0, pid: pid, c: c },
                    DocOp::Delete(pid) => SessionMessage::Delete { site: 0, pid: pid }
                };
                
                if ws.send(Message::from(msg.serialize())).is_ok() {
                    queue.pop_front();
                }
            }
        }
    }
    // pub fn flush_all_to_server(&mut self, ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    //     for (doc_id, queue) in self.log.iter_mut() {
    //         if let Some(op) = queue.front() {
    //             let mut buf = Vec::new();
    //             op.serialize_into(&mut buf);
    //             if ws.send(Message::from(buf)).is_ok() {
    //                 queue.pop_front();
    //                 // wait for ack before popping
    //             }
    //         }
    //     }
    // }
    // /// Append current in-memory oplog to the on-disk oplog file, then clear memory.
    // pub fn flush_oplog(&mut self) -> Result<()> {
    //     if self.log.is_empty() {
    //         return Ok(());
    //     }
    //     let path = self.get_oplog_path();
    //     if let Some(parent) = path.parent() {
    //         fs::create_dir_all(parent)?;
    //     }
    //     let file = fs::OpenOptions::new()
    //         .create(true)
    //         .append(true)
    //         .open(&path)?;
    //     let mut writer = BufWriter::new(file);
    //     for op in &self.log {
    //         op.write_to(&mut writer)?;
    //     }
    //     writer.flush()?;
    //     self.log.clear();
    //     Ok(())
    // }
    //
    // /// Load oplog from disk. Returns empty vec if file doesn't exist or is unreadable.
    // fn load_oplog(name: &Path) -> Vec<DocOp> {
    //     let path = hidden_oplog_path(name);
    //     let file = match File::open(&path) {
    //         Ok(f) => f,
    //         Err(_) => return Vec::new(),
    //     };
    //     let mut reader = BufReader::new(file);
    //     let mut ops = Vec::new();
    //     loop {
    //         match DocOp::read_from(&mut reader) {
    //             Ok(op) => ops.push(op),
    //             Err(_) => break,
    //         }
    //     }
    //     ops
    // }

    // /// Drain all ops from the oplog, returning them and clearing both memory and disk.
    // pub fn drain_oplog(&mut self) -> Result<Vec<DocOp>> {
    //     // First flush any in-memory ops to disk so we capture everything
    //     self.flush_oplog()?;
    //     // Reload from disk to get the full set
    //     let ops = Self::load_oplog(&self.name);
    //     // Clear the on-disk file
    //     self.clear_oplog_file()?;
    //     Ok(ops)
    // }
    //
    // /// Remove the oplog file from disk.
    // fn clear_oplog_file(&self) -> Result<()> {
    //     let path = self.get_oplog_path();
    //     if path.exists() {
    //         fs::remove_file(&path)?;
    //     }
    //     Ok(())
    // }
    //
    // fn get_oplog_path(&self) -> PathBuf {
    //     hidden_oplog_path(&self.name)
    // }
}
