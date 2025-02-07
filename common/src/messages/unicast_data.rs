use std::io::{self, Cursor};

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct UnicastData {
    pub client_id: String,
    pub topic: String,
    pub data_packets: Vec<DataPacket>,
}

impl UnicastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::UnicastData
    }
}

impl Serializable for UnicastData {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(UnicastData {
            client_id: String::deserialize(reader)?,
            topic: String::deserialize(reader)?,
            data_packets: Vec::<DataPacket>::deserialize(reader)?,
        })
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.client_id.serialize(writer)?;
        self.topic.serialize(writer)?;
        self.data_packets.serialize(writer)?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.client_id.size() + self.topic.size() + self.data_packets.size()
    }
}
