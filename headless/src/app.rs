use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use algos::session::SessionMessage;

use crate::editor_message::EditorMessage;
use crate::state::State;

pub enum AppEvent {
    FileCreated(PathBuf),
    FileRenamed { from: PathBuf, to: PathBuf },
    EditorMsg(EditorMessage),
    ClientDisconnected,
}

pub fn run_app(rx: Receiver<AppEvent>, state: &mut State, session_tx: Sender<SessionMessage>) {
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
                // TODO use the sync protocol instead of session one
                let msg = SessionMessage::Start {
                    document_id: doc.id,
                    last_sync_time: 0,
                    name: Some(path),
                };
                let _ = session_tx.send(msg);
                for (k, v) in doc.get_doc().content.iter() {
                    let msg = SessionMessage::Insert {
                        site: 0,
                        pid: k.clone(),
                        c: v.0,
                    };
                    let _ = session_tx.send(msg);
                }
            }
            AppEvent::FileRenamed { from, to } => {
                state.move_doc(from, to.clone());
                // TODO use the sync protocol/some other call as start isn't meant to really do that
                // but change name is missing the id and it's also a session call
                let doc_id = state.get_doc_by_name(&to).id;
                let msg = SessionMessage::Start {
                    document_id: doc_id,
                    last_sync_time: 0,
                    name: Some(to),
                };
                let _ = session_tx.send(msg);
            }
            AppEvent::EditorMsg(msg) => match msg {
                EditorMessage::ChooseDocument(doc_name) => {
                    state.set_current_doc(&doc_name);
                    let msg = SessionMessage::Start {
                        document_id: state.get_current_doc_id(),
                        last_sync_time: 0,
                        name: None,
                    };
                    let _ = session_tx.send(msg);
                }
                EditorMessage::Insert(pos, text) => {
                    println!("Text received {} {}", pos, text);
                    let inserted = state.insert_in_current_doc(pos, &text);
                    for (pid, c) in inserted {
                        let msg = SessionMessage::Insert { site: 0, pid, c };
                        let _ = session_tx.send(msg);
                    }
                }
                EditorMessage::Delete(start, len) => {
                    println!("Text deleted from range: {} {}", start, len);
                    let deleted = state.delete_in_current_doc(start, len);
                    for pid in deleted {
                        let msg = SessionMessage::Delete { site: 0, pid };
                        let _ = session_tx.send(msg);
                    }
                }
                EditorMessage::Flush => {
                    let _ = state.flush_current_doc();
                }
            },
            AppEvent::ClientDisconnected => {
                println!("Client disconnected");
            }
        }
    }
}
