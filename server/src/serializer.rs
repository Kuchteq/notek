
use algos::PeerMessage;
use tokio_tungstenite::tungstenite::Message;

pub enum SerializeFormat {
    Bincode,
    MsgPack,
    Mine
}

pub struct Serializer {
    format: SerializeFormat,
}

impl Serializer {
    pub fn new(format: SerializeFormat) -> Serializer {
        Serializer { format }
    }
    pub fn serialize(&self, msg: &PeerMessage) -> Message {
        match self.format {
            SerializeFormat::Bincode => {
                let bytes = bincode::serialize(&msg).unwrap();
                Message::from(bytes)
            }
            SerializeFormat::MsgPack => {
                let bytes = rmp_serde::to_vec(&msg).unwrap();
                Message::from(bytes)
            }
            SerializeFormat::Mine => {
                let bytes = msg.serialize();
                Message::from(bytes)
            }
        }
    }
    pub fn deserialize(&self, msg: &[u8]) -> PeerMessage {
        match self.format {
            SerializeFormat::Bincode => {
                        let bytes = bincode::deserialize(&msg).unwrap();
                        PeerMessage::from(bytes)
                    }
            SerializeFormat::MsgPack => {
                        let bytes = rmp_serde::from_slice(msg).unwrap();
                        PeerMessage::from(bytes)
                    }
            SerializeFormat::Mine => {
                PeerMessage::deserialize(msg)
            }
        }
    }
}
