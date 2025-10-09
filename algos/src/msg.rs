use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{Doc, Pid};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum PeerMessage {
    Greet,
    Insert(u8, Pid, char),
    Delete(u8, Pid),

    // client only
    NewSession(u8, Doc),
    NewSessionRaw { site: u8, keys: Vec<Pid>, values: Vec<char> },
}
