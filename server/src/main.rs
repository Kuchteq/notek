use algos::doc::Doc;
use algos::msg::PeerMessage;
use algos::pid::Pid;
use rand::Rng;
use std::{fs, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::serializer::Serializer;
mod serializer;

enum DocCommand {
    Insert(u8, Pid, char),
    Delete(u8, Pid),
    GetSnapshot(oneshot::Sender<Arc<Doc>>),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    println!("Listening on 127.0.0.1:9001");

    let (dcmd_tx, mut dcmd_rx) = mpsc::unbounded_channel::<DocCommand>();
    let (bcast_tx, _) = broadcast::channel::<PeerMessage>(100);

    tokio::spawn({
        let bcast_tx = bcast_tx.clone();
        async move {
            let mut doc = Arc::new(Doc::new("Hello world".to_string()));
            while let Some(cmd) = dcmd_rx.recv().await {
                match cmd {
                    DocCommand::Insert(site, pid, c) => {
                        // doc.insert(pid, c);
                        Arc::make_mut(&mut doc).insert(pid.clone(), c); // clone-on-write
                        let _ = bcast_tx.send(PeerMessage::Insert{ site, pid, c });
                    }
                    DocCommand::Delete(site, pid) => {
                        Arc::make_mut(&mut doc).delete(&pid);
                        let _ = bcast_tx.send(PeerMessage::Delete{ site, pid });
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
    let connection_site_id = rand::rng().random_range(0..255);
    let (mut ws_sink, mut ws_stream) = ws.split();
    let serializer = Serializer::new(serializer::SerializeFormat::Mine);

    println!(
        "New WebSocket connection, assigned id: {}",
        connection_site_id
    );

    loop {
        tokio::select! {
            Some(msg) = ws_stream.next() => {
                    let msg = msg?;
                    if let Message::Binary(bin) = msg {

                        let msg: PeerMessage = serializer.deserialize(&bin);
                        match msg {
                            PeerMessage::Insert{ site, pid, c } => {
                                dcmd_tx.send(DocCommand::Insert(site, pid, c));
                            }
                            PeerMessage::Delete{ site, pid } => {
                                dcmd_tx.send(DocCommand::Delete(site, pid));
                            }
                            PeerMessage::Greet{} => {
                                let (resp_tx, resp_rx) = oneshot::channel();
                                dcmd_tx.send(DocCommand::GetSnapshot(resp_tx)).unwrap();
                                let snapshot = resp_rx.await.unwrap();
                                let response = PeerMessage::NewSession{ site: connection_site_id, doc: (*snapshot).clone() };
                                let msg = serializer.serialize(&response);
                                ws_sink.send(msg).await?;
                            }
                            _ => {}
                        }
                    }
            }
        Ok(update) = bcast_rx.recv() => {
                let should_receive = match &update {
                    PeerMessage::Insert{site, ..} => { println!("{}", site);
                        *site != connection_site_id},
                    PeerMessage::Delete{site, ..} => *site != connection_site_id,
                    _ => false
                };
                if !should_receive {
                    continue
                }
                let bytes = serializer.serialize(&update);
                if ws_sink.send(Message::from(bytes)).await.is_err() {
                    break;
                }
            }
        }
    }

    Ok(())
}
