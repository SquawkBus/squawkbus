use std::io;

use crate::frame::{FrameReader, FrameWriter};
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

    pub fn read(reader: &mut FrameReader) -> io::Result<MulticastData> {
        Ok(MulticastData {
            topic: String::deserialize(reader)?,
            data_packets: Vec::<DataPacket>::deserialize(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.topic).serialize(writer)?;
        (&self.data_packets).serialize(writer)?;
        Ok(())
    }
}
