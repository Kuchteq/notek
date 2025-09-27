use algos::Doc;
use crossterm::event::{self, Event, KeyCode};
use futures::{SinkExt, StreamExt};
use ratatui::{
    Terminal,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::protocol::Message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (mut ws_stream, _) = connect_async("ws://127.0.0.1:9001").await?;

    // Send a message
    // ws_stream.send(Message::Text("Hello WebSocket!".into())).await?;

    // Receive a message
    let mut d: Doc = if let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Binary(bin) => bincode::deserialize(&bin).unwrap(),
            _ => Doc::new(String::new()),
        }
    } else {
        Doc::new(String::new())
    };

    let mut terminal = ratatui::init();

    // Editor state
    let mut cursor = (0usize, 0usize); // (row, column)

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

        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    let row = cursor.0;
                    let col = cursor.1;
                    d.insert_idx(col, c);
                    // content[row].insert(col, c);
                    cursor.1 += 1;
                }
                KeyCode::Enter => {
                    let row = cursor.0;
                    let col = cursor.1;
                    let new_line = content[row].split_off(col);
                    content.insert(row + 1, new_line);
                    cursor.0 += 1;
                    cursor.1 = 0;
                }
                KeyCode::Backspace => {
                    let row = cursor.0;
                    let col = cursor.1;
                    if col > 0 {
                        content[row].remove(col - 1);
                        cursor.1 -= 1;
                    } else if row > 0 {
                        let prev_len = content[row - 1].len();
                        let line = content.remove(row);
                        content[row - 1].push_str(&line);
                        cursor.0 -= 1;
                        cursor.1 = prev_len;
                    }
                }
                KeyCode::Esc => break,
                KeyCode::Left => {
                    if cursor.1 > 0 {
                        cursor.1 -= 1;
                    } else if cursor.0 > 0 {
                        cursor.0 -= 1;
                        cursor.1 = content[cursor.0].len();
                    }
                }
                KeyCode::Right => {
                    if cursor.1 < content[cursor.0].len() {
                        cursor.1 += 1;
                    } else if cursor.0 + 1 < content.len() {
                        cursor.0 += 1;
                        cursor.1 = 0;
                    }
                }
                KeyCode::Up => {
                    if cursor.0 > 0 {
                        cursor.0 -= 1;
                        cursor.1 = cursor.1.min(content[cursor.0].len());
                    }
                }
                KeyCode::Down => {
                    if cursor.0 + 1 < content.len() {
                        cursor.0 += 1;
                        cursor.1 = cursor.1.min(content[cursor.0].len());
                    }
                }
                _ => {}
            }
        }
    }

    ratatui::restore();

    Ok(())
}
