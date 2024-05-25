use std::io::prelude::*;
use std::io;

use crate::common::serialization::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct ForwardedMulticastData {
    pub host: String,
    pub user: String,
    pub feed: String,
    pub topic: String,
    pub content_type: String,
    pub data_packets: Vec<DataPacket>
}

impl ForwardedMulticastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedMulticastData
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<ForwardedMulticastData> {
        Ok(ForwardedMulticastData {
            host: String::read(&mut reader)?,
            user: String::read(&mut reader)?,
            feed: String::read(&mut reader)?,
            topic: String::read(&mut reader)?,
            content_type: String::read(&mut reader)?,
            data_packets: Vec::<DataPacket>::read(&mut reader)?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.host).write(&mut writer)?;
        (&self.user).write(&mut writer)?;
        (&self.feed).write(&mut writer)?;
        (&self.topic).write(&mut writer)?;
        (&self.content_type).write(&mut writer)?;
        (&self.data_packets).write(&mut writer)?;
        Ok(())
    }
}
