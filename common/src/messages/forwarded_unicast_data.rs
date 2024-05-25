use std::io::prelude::*;
use std::io;

use uuid::Uuid;

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct ForwardedUnicastData {
    pub host: String,
    pub user: String,
    pub client_id: Uuid,
    pub feed: String,
    pub topic: String,
    pub content_type: String,
    pub data_packets: Vec<DataPacket>
}

impl ForwardedUnicastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedUnicastData
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<ForwardedUnicastData> {
        Ok(ForwardedUnicastData {
            host: String::read(&mut reader)?,
            user: String::read(&mut reader)?,
            client_id: Uuid::read(&mut reader)?,
            feed: String::read(&mut reader)?,
            topic: String::read(&mut reader)?,
            content_type: String::read(&mut reader)?,
            data_packets: Vec::<DataPacket>::read(&mut reader)?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.host).write(&mut writer)?;
        (&self.user).write(&mut writer)?;
        (&self.client_id).write(&mut writer)?;
        (&self.feed).write(&mut writer)?;
        (&self.topic).write(&mut writer)?;
        (&self.content_type).write(&mut writer)?;
        (&self.data_packets).write(&mut writer)?;
        Ok(())
    }
}
