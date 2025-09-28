use algos::{Doc, PeerMessage, Pid};

use futures::{Sink, Stream};
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::{self, protocol::Message};

use crate::events::AppEvent;

pub enum RemoteEvent {
    InsertAt(Pid, char),
    DeleteAt(Pid),
}

// Handle an incoming WebSocket message and send an internal event
pub async fn handle_incoming(bin: &[u8], ev_tx: &tokio::sync::mpsc::UnboundedSender<AppEvent>) {
    let msg: PeerMessage = bincode::deserialize(bin).unwrap();
    match msg {
        PeerMessage::Insert(pid, c) => {
            let _ = ev_tx.send(AppEvent::InsertAt(pid, c));
        }
        PeerMessage::Delete(pid) => {
            let _ = ev_tx.send(AppEvent::DeleteAt(pid));
        }
    }
}

// Handle an outgoing event and send it over the WebSocket sink
pub async fn handle_outgoing<S>(ev: RemoteEvent, ws_sink: &mut S) -> Result<(), S::Error>
where
    S: Sink<Message> + Unpin,
{
    let msg = match ev {
        RemoteEvent::InsertAt(pid, c) => PeerMessage::Insert(pid, c),
        RemoteEvent::DeleteAt(pid) => PeerMessage::Delete(pid),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    ws_sink.send(Message::from(bytes)).await
}


pub async fn greet<S, R>(ws_sink: &mut S, ws_stream: &mut R) -> Doc
where
    S: Sink<Message> + Unpin,
    R: Stream<Item = Result<Message, tungstenite::Error>> + Unpin,
{
    // Send a message
    // let bytes = bincode::serialize(&PeerMessage::Greet(1)).unwrap();
    // ws_sink.send(Message::from(bytes)).await.unwrap();
    //
    // // Receive a message
    // let d: Doc = match ws_stream.next().await {
    //     Some(Ok(Message::Binary(bin))) => bincode::deserialize(&bin).unwrap(),
    //     _ => Doc::new(String::new()),
    // };
    //
    d
}
