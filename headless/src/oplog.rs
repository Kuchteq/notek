use std::{
    collections::{BTreeMap, VecDeque},
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    sync::mpsc::{Receiver, Sender},
    time::Duration,
    u128,
};

use algos::{
    pid::Pid,
    session::SessionMessage,
    sync::{DocOp, SyncRequests},
};
use anyhow::Result;
use tungstenite::{Message, WebSocket, connect, stream::MaybeTlsStream};

pub struct Oplog {
    pub current_document: u128,
    pub current_log: VecDeque<DocOp>,
    pub log: BTreeMap<u128, VecDeque<DocOp>>,
    pub session_available: bool,
    pub sync_available: bool,
}

pub enum OplogMsg {
    SessionMessage(SessionMessage),
    SyncAvailable,
    SyncDown,
    SessionAvailable,
    SessionDown,
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
            current_log: VecDeque::new(),
            log: BTreeMap::new(),
            session_available: false,
            sync_available: false,
        })
    }

    pub fn run(
        &mut self,
        rx: Receiver<OplogMsg>,
        sync_tx: Sender<SyncRequests>,
        session_tx: Sender<SessionMessage>,
    ) {
        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(event) => match event {
                    OplogMsg::SessionMessage(msg) => {
                        match msg {
                            SessionMessage::Insert { pid, c, .. } => {
                                if self.session_available {
                                    session_tx.send(SessionMessage::Insert { site: 0, pid, c });
                                } else {
                                    self.current_log.push_back(DocOp::Insert(pid, c));
                                }
                            }
                            SessionMessage::Start { document_id, .. } => {
                                if document_id != self.current_document {
                                    if self.current_document != u128::MAX
                                        && self.current_log.len() > 0
                                    {
                                        self.log.insert(
                                            self.current_document,
                                            std::mem::take(&mut self.current_log),
                                        );
                                    }
                                    // If we already have some previous oplog existing for the
                                    // document we started editing, then take it from the current
                                    // one
                                    if let Some(l) = self.log.remove(&document_id) {
                                        self.current_log = l;
                                    }

                                    self.current_document = document_id;
                                    if self.session_available && self.current_document != u128::MAX
                                    {
                                        session_tx.send(msg);
                                    }
                                }
                                // self.log.insert(document_id, VecDeque::new());
                            }
                            SessionMessage::Delete { site, pid } => {
                                if self.session_available {
                                    session_tx.send(SessionMessage::Delete { site, pid });
                                } else {
                                    self.current_log.push_back(DocOp::Delete(pid));
                                }
                            }
                            SessionMessage::ChangeName { name } => todo!(),
                        }
                        // let log = self.log.get_mut(&self.current_document).unwrap();
                        // log.push_back(DocOp::Insert(pid, c));
                    }
                    OplogMsg::SessionAvailable => {
                        self.session_available = true;
                        if self.current_document != u128::MAX {
                            session_tx.send(SessionMessage::Start {
                                document_id: self.current_document,
                                last_sync_time: 0,
                                name: None,
                            });
                        }
                    }
                    OplogMsg::SyncAvailable => {
                        for (did, l) in std::mem::take(&mut self.log) {
                            let mut inserts = Vec::new();
                            let mut deletes = Vec::new();

                            for item in l {
                                match item {
                                    DocOp::Insert(pid, c) => inserts.push((pid, c)),
                                    DocOp::Delete(pid) => deletes.push(pid),
                                    _ => {}
                                }
                            }

                            let req = SyncRequests::SyncDocUpsert {
                                document_id: did,
                                name: None,
                                last_sync_time: 0,
                                inserts: inserts,
                                deletes: deletes,
                            };
                            sync_tx.send(req);
                        }
                    }
                    OplogMsg::SyncDown => todo!(),
                    OplogMsg::SessionDown => {
                        self.session_available = false;
                    }
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // no message → fall through to sending
                }
                Err(_) => break,
            }

            // self.flush_to_server(&mut ws);
        }
    }

    // pub fn handle_start(&mut self, document_id: u128) {}
    pub fn flush_to_server(&mut self, ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
        if let Some(queue) = self.log.get_mut(&self.current_document) {
            if let Some(op) = queue.pop_front() {
                let msg = match op {
                    DocOp::Insert(pid, c) => SessionMessage::Insert {
                        site: 0,
                        pid: pid,
                        c: c,
                    },
                    DocOp::Delete(pid) => SessionMessage::Delete { site: 0, pid: pid },
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
