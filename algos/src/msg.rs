use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{Doc, Pid};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PeerMessage {
    Greet,
    NewSession(u8, Doc),
    Insert(Pid, char),
    Delete(Pid)
}
