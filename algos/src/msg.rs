use serde::{Deserialize, Serialize};

use crate::Pid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PeerMessage {
    Greet(u8),
    Insert(Pid, char),
    Delete(Pid)
}
