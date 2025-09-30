use algos::{Doc, PeerMessage, Pid};
use bincode::{DefaultOptions, config};
use std::{fs, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

// fn main() {
//     let content = fs::read_to_string("foo.txt").unwrap();
//     let doc = Doc::new(content.to_string());
//     println!("{:#?}", doc.len());
// }
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{accept_async, tungstenite::Message};

enum DocCommand {
    Insert(Pid, char),
    Delete(Pid),
    GetSnapshot(oneshot::Sender<Arc<Doc>>),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9001").await?;
    println!("Listening on 127.0.0.1:9001");

    let (dcmd_tx, mut dcmd_rx) = mpsc::unbounded_channel::<DocCommand>();
    let (bcast_tx, _) = broadcast::channel::<PeerMessage>(100);

    tokio::spawn({
        let bcast_tx = bcast_tx.clone();
        async move {
            let mut doc = Arc::new(Doc::new("Hello world".to_string()));
            while let Some(cmd) = dcmd_rx.recv().await {
                match cmd {
                    DocCommand::Insert(pid, c) => {
                        // doc.insert(pid, c);
                        Arc::make_mut(&mut doc).insert(pid.clone(), c); // clone-on-write
                        let _ = bcast_tx.send(PeerMessage::Insert(pid, c));
                    }
                    DocCommand::Delete(pid) => {
                        Arc::make_mut(&mut doc).delete(&pid);
                        let _ = bcast_tx.send(PeerMessage::Delete(pid));
                    }
                    DocCommand::GetSnapshot(resp) => {
                        let _ = resp.send(doc.clone());
                    }
                }
                println!("{:#?}", doc.to_string());
            }
        }
    });

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(
            stream,
            dcmd_tx.clone(),
            bcast_tx.subscribe(),
        ));
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    dcmd_tx: UnboundedSender<DocCommand>,
    mut bcast_rx: broadcast::Receiver<PeerMessage>,
) -> anyhow::Result<()> {
    let ws = accept_async(stream).await?;
    let connection_site_id = 1;
    let (mut ws_sink, mut ws_stream) = ws.split();

    println!("New WebSocket connection");

    tokio::select! {
        Some(msg) = ws_stream.next() => {
                let msg = msg?;
                if let Message::Binary(bin) = msg {
                    let msg: PeerMessage = bincode::deserialize(&bin).unwrap();
                    match msg {
                        PeerMessage::Insert(pid, c) => {
                            dcmd_tx.send(DocCommand::Insert(pid, c));
                        }
                        PeerMessage::Delete(pid) => {
                            dcmd_tx.send(DocCommand::Delete(pid));
                        }
                        PeerMessage::Greet => {
                            let (resp_tx, resp_rx) = oneshot::channel();
                            dcmd_tx.send(DocCommand::GetSnapshot(resp_tx)).unwrap();
                            let snapshot = resp_rx.await.unwrap();
                            let response = PeerMessage::NewSession(connection_site_id, (*snapshot).clone());
                            let bytes = bincode::serialize(&response).unwrap();
                            let msg = Message::from(bytes);

                            ws_sink.send(msg).await?;
                        }
                        PeerMessage::NewSession(_, _) => {}
                    }
                }
        }
        Ok(update) = bcast_rx.recv() => {
                let bytes = bincode::serialize(&update).unwrap();
                if ws_sink.send(Message::from(bytes)).await.is_err() {
                }
            }

    }

    Ok(())
}
