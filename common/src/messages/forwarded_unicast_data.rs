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
            host: String::deserialize(reader)?,
            user: String::deserialize(reader)?,
            client_id: String::deserialize(reader)?,
            topic: String::deserialize(reader)?,
            data_packets: Vec::<DataPacket>::deserialize(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.host).serialize(writer)?;
        (&self.user).serialize(writer)?;
        (&self.client_id).serialize(writer)?;
        (&self.topic).serialize(writer)?;
        (&self.data_packets).serialize(writer)?;
        Ok(())
    }
}
