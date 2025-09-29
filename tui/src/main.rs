use algos::{Doc, PeerMessage, Pid, Pos};
use crossterm::event::{self, Event, KeyCode, read};
use futures::{SinkExt, StreamExt};
use ratatui::{
    Terminal,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use serde::Deserialize;
use tokio::{select, sync::mpsc};
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::protocol::Message;

use crate::{
    events::{handle_event, interpret_key, AppEvent},
    remote::{greet, handle_incoming, handle_outgoing, RemoteEvent},
};
mod events;
mod remote;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (ws, _) = connect_async("ws://127.0.0.1:9001").await?;

    let (mut ws_sink, mut ws_stream) = ws.split();
    // Send a message
    // ws_stream.send(Message::Text("Hello WebSocket!".into())).await?;

    // Receive a message
    // let mut d: Doc = if let Some(msg) = ws_stream.next().await {
    //     match msg? {
    //         Message::Binary(bin) => bincode::deserialize(&bin).unwrap(),
    //         _ => Doc::new(String::new()),
    //     }
    // } else {
    //     Doc::new(String::new())
    // };
    greet(&mut ws_sink, &mut ws_stream).await;
    let mut d = Doc::new("hi".to_string());

    // Main app event channel
    let (ev_tx, mut ev_rx) = mpsc::unbounded_channel::<AppEvent>();

    // input handling
    tokio::spawn({
        let ev_tx = ev_tx.clone();
        async move {
            loop {
                // crossterm::event::read() is blocking, so spawn in blocking thread
                if let Ok(event) = tokio::task::spawn_blocking(|| crossterm::event::read()).await {
                    if let Ok(crossterm::event::Event::Key(key)) = event {
                        let ev = interpret_key(key);
                        let should_stop = matches!(ev, AppEvent::Quit);
                        ev_tx.send(ev);
                        if should_stop {
                            break;
                        }
                        // let _ = input_tx.send(InputEvent::Key(key));
                    }
                }
            }
        }
    });

    let (rm_tx, mut rm_rx) = mpsc::unbounded_channel::<RemoteEvent>();

    // remote events
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = ws_stream.next() => {
                    if let Ok(Message::Binary(bin)) = msg {
                        handle_incoming(&bin, &ev_tx).await;
                    }
                }

                Some(ev) = rm_rx.recv() => {
                    let _ = handle_outgoing(ev, &mut ws_sink).await;
                }
            }
        }
    });

    let mut terminal = ratatui::init();

    let mut cursorng = Pid::new(0); // (row, column)
    // Editor state
    loop {
        // Draw UI
        let mut content = vec![d.to_string()]; // lines of text
        terminal.draw(|f| {
            let size = f.area();

            let block = Block::default()
                .borders(Borders::ALL)
                .title("Ratatui Editor");
            let paragraph = Paragraph::new(content.join("\n")).block(block);
            f.render_widget(paragraph, size);
        })?;

        select! {
            Some(ev) = ev_rx.recv() => {
                let finished = handle_event(ev, &mut d, &mut cursorng, &rm_tx);
                if finished {
                    break
                }
            }
        }
    }

    ratatui::restore();

    Ok(())
}
