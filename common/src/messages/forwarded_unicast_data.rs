use std::io;

use crate::frame::{FrameReader, FrameWriter};
use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct ForwardedUnicastData {
    pub host: String,
    pub user: String,
    pub client_id: String,
    pub topic: String,
    pub data_packets: Vec<DataPacket>,
}

impl ForwardedUnicastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedUnicastData
    }

    pub fn read(reader: &mut FrameReader) -> io::Result<ForwardedUnicastData> {
        Ok(ForwardedUnicastData {
            host: String::read(reader)?,
            user: String::read(reader)?,
            client_id: String::read(reader)?,
            topic: String::read(reader)?,
            data_packets: Vec::<DataPacket>::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.host).write(writer)?;
        (&self.user).write(writer)?;
        (&self.client_id).write(writer)?;
        (&self.topic).write(writer)?;
        (&self.data_packets).write(writer)?;
        Ok(())
    }
}
