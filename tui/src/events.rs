use algos::{doc::Doc, pid::Pid};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::remote::RemoteEvent;

pub enum AppEvent {
    NewSession(u8, Doc),
    CursorInsert(char),
    CursorDelete,
    CursorMove(isize),

    InsertAt(Pid, char),
    DeleteAt(Pid),
    MoveTo(Pid),
    Skip,
    Quit,
}

pub fn interpret_key(key: KeyEvent) -> AppEvent {
    match key.code {
        KeyCode::Char(c) => {
            AppEvent::CursorInsert(c)
            // cursorng.0 = d.insert_left(cursorng.0.clone(), c);
            // let msg = algos::PeerMessage::Insert(cursorng.0.clone(), c);
            // let bytes = bincode::serialize(&msg).unwrap();
            // ws_sink.send(Message::from(bytes)).await;
        }
        KeyCode::Backspace => {
            AppEvent::CursorDelete
            // let new_place = d.left(&cursorng.0);
            // d.delete(&cursorng.0);
            // let msg = algos::PeerMessage::Delete(cursorng.0.clone());
            // let bytes = bincode::serialize(&msg).unwrap();
            // ws_sink.send(Message::from(bytes)).await;
            // cursorng.0 = new_place;
        }
        KeyCode::Esc => AppEvent::Quit,
        KeyCode::Left => {
            AppEvent::CursorMove(1)
            // cursorng.0 = d.left(&cursorng.0);
        }
        _ => AppEvent::Skip
    }
}

pub fn handle_event(ev : AppEvent, doc: &mut Doc, cursor: &mut Pid, rm_tx: &UnboundedSender<RemoteEvent>, site: &mut u8) -> bool {
    match ev {
        AppEvent::CursorInsert(c) => 
                {
                    *cursor = doc.insert_left(cursor.clone(), c);
                    rm_tx.send(RemoteEvent::InsertAt(*site, cursor.clone(), c));
                }
                    ,
        AppEvent::CursorDelete => { 
                    let new_place = doc.left(&cursor);
                    doc.delete(&cursor);
                    rm_tx.send(RemoteEvent::DeleteAt(*site, cursor.clone()));
                    *cursor = new_place;

                },
        AppEvent::CursorMove(off) => { *cursor = doc.offset(cursor, off).unwrap() },
        AppEvent::InsertAt(pid, c) => {
                    doc.insert(pid, c);
                },
        AppEvent::DeleteAt(pid) => {
                    doc.delete(&pid);
                },
        AppEvent::MoveTo(pid) => todo!(),
        AppEvent::Skip => (),
        AppEvent::Quit => return true,
        AppEvent::NewSession(s, new_doc) => {*doc = new_doc; *site = s },
    };
    false
}
