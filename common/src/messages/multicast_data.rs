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
            topic: String::read(reader)?,
            data_packets: Vec::<DataPacket>::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.topic).write(writer)?;
        (&self.data_packets).write(writer)?;
        Ok(())
    }
}
