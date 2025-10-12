
use algos::msg::PeerMessage;
use algos::pid::Pid;
use futures::{Sink, Stream};
use futures::SinkExt;
use tokio_tungstenite::tungstenite::{self, protocol::Message};

use crate::events::AppEvent;
use futures::StreamExt;

pub enum RemoteEvent {
    InsertAt(u8, Pid, char),
    DeleteAt(u8, Pid),
}

// Handle an incoming WebSocket message and send an internal event
pub async fn handle_incoming(bin: &[u8], ev_tx: &tokio::sync::mpsc::UnboundedSender<AppEvent>) {
    let msg: PeerMessage = PeerMessage::deserialize(bin);
    match msg {
        PeerMessage::Insert{site, pid, c} => {
                    let _ = ev_tx.send(AppEvent::InsertAt(pid, c));
                }
        PeerMessage::Delete{site, pid} => {
                    let _ = ev_tx.send(AppEvent::DeleteAt(pid));
                }
        PeerMessage::Greet => {},
        PeerMessage::NewSession{site, doc} => { ev_tx.send(AppEvent::NewSession(site, doc)); }
    }
}

// Handle an outgoing event and send it over the WebSocket sink
pub async fn handle_outgoing<S>(ev: RemoteEvent, ws_sink: &mut S) -> Result<(), S::Error>
where
    S: Sink<Message> + Unpin,
{
    let msg = match ev {
        RemoteEvent::InsertAt(site, pid, c) => PeerMessage::Insert{site, pid, c},
        RemoteEvent::DeleteAt(site, pid) => PeerMessage::Delete{site, pid},
    };
    let bytes = msg.serialize();
    ws_sink.send(Message::from(bytes)).await
}


pub async fn greet<S, R>(ws_sink: &mut S, ws_stream: &mut R)
where
    S: Sink<Message> + Unpin,
    R: Stream<Item = Result<Message, tungstenite::Error>> + Unpin,
{
    
    let bytes = PeerMessage::Greet.serialize();
    ws_sink.send(Message::from(bytes)).await;
}
