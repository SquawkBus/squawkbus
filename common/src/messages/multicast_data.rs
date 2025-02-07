use std::io;

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct MulticastData {
    pub topic: String,
    pub data_packets: Vec<DataPacket>,
}

impl MulticastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::MulticastData
    }
}

impl Serializable for MulticastData {
    fn deserialize(reader: &mut io::Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(MulticastData {
            topic: String::deserialize(reader)?,
            data_packets: Vec::<DataPacket>::deserialize(reader)?,
        })
    }

    fn serialize(&self, writer: &mut io::Cursor<Vec<u8>>) -> io::Result<()> {
        self.topic.serialize(writer)?;
        self.data_packets.serialize(writer)?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.topic.size() + self.data_packets.size()
    }
}
