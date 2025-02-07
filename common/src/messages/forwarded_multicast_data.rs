use std::io::{self, Cursor};

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct ForwardedMulticastData {
    pub host: String,
    pub user: String,
    pub topic: String,
    pub data_packets: Vec<DataPacket>,
}

impl ForwardedMulticastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedMulticastData
    }
}

impl Serializable for ForwardedMulticastData {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<ForwardedMulticastData> {
        Ok(ForwardedMulticastData {
            host: String::deserialize(reader)?,
            user: String::deserialize(reader)?,
            topic: String::deserialize(reader)?,
            data_packets: Vec::<DataPacket>::deserialize(reader)?,
        })
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.host.serialize(writer)?;
        self.user.serialize(writer)?;
        self.topic.serialize(writer)?;
        self.data_packets.serialize(writer)?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.host.size() + self.user.size() + self.topic.size() + self.data_packets.size()
    }
}
