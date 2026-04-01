use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use algos::session::SessionMessage;
use algos::sync::SyncRequests;

use crate::editor_message::EditorMessage;
use crate::oplog::OplogMsg;
use crate::state::{ConnectionStatus, State};

pub enum AppEvent {
    FileCreated(PathBuf),
    FileRenamed { from: PathBuf, to: PathBuf },
    EditorMsg(EditorMessage),
    ClientDisconnected,
    SyncConnected,
    SyncDisconnected,
    SessionConnected,
    SessionDisconnected,
}

pub fn run_app(
    rx: Receiver<AppEvent>,
    state: &mut State,
    oplog_tx: Sender<OplogMsg>,
    sync_tx: Sender<SyncRequests>,
) {
    // Main event loop — State stays here, single-threaded mutations
    while let Ok(event) = rx.recv() {
        match event {
            AppEvent::FileCreated(path) => {
                println!("Adding new document: {:?}", path);
                let doc = match state.add_doc(path.clone(), None) {
                    Ok(doc) => doc,
                    Err(e) => {
                        eprintln!("Failed to add document: {}", e);
                        continue;
                    }
                };
                let inserts: Vec<_> = doc
                    .get_doc()
                    .content
                    .iter()
                    .map(|(k, v)| (k.clone(), v.0))
                    .collect();
                let msg = SyncRequests::SyncDocUpsert {
                    document_id: doc.id,
                    name: Some(path),
                    last_sync_time: 0,
                    inserts,
                    deletes: Vec::new(),
                };
                let _ = sync_tx.send(msg);
            }
            AppEvent::FileRenamed { from, to } => {
                state.move_doc(from, to.clone());
                let doc_id = state.get_doc_by_name(&to).id;
                let msg = SyncRequests::DocNameChange {
                    document_id: doc_id,
                    name: to,
                };
                let _ = sync_tx.send(msg);
            }
            AppEvent::EditorMsg(msg) => match msg {
                EditorMessage::ChooseDocument(doc_name) => {
                    state.set_current_doc(&doc_name);
                    let msg = SessionMessage::Start {
                        document_id: state.get_current_doc_id(),
                        last_sync_time: 0,
                        name: None,
                    };
                    let _ = oplog_tx.send(OplogMsg::SessionMessage(msg));
                }
                EditorMessage::Insert(pos, text) => {
                    println!("Text received {} {}", pos, text);
                    let inserted = state.insert_in_current_doc(pos, &text);
                    for (pid, c) in inserted {
                        let msg = SessionMessage::Insert { site: 0, pid, c };
                        let _ = oplog_tx.send(OplogMsg::SessionMessage(msg));
                    }
                }
                EditorMessage::Delete(start, len) => {
                    println!("Text deleted from range: {} {}", start, len);
                    let deleted = state.delete_in_current_doc(start, len);
                    for pid in deleted {
                        let msg = SessionMessage::Delete { site: 0, pid };
                        let _ = oplog_tx.send(OplogMsg::SessionMessage(msg));
                    }
                }
                EditorMessage::Flush => {
                    let _ = state.flush_current_doc();
                }
            },
            AppEvent::ClientDisconnected => {
                println!("Client disconnected");
            }
            AppEvent::SyncConnected => {
                println!("Connected to sync server");
            }
            AppEvent::SyncDisconnected => {
                println!("Disconnected from sync server");
            }
            AppEvent::SessionConnected => {
                oplog_tx.send(OplogMsg::SessionAvailable);
            },
            AppEvent::SessionDisconnected => {
                oplog_tx.send(OplogMsg::SessionDown);
            },
        }
    }
}
