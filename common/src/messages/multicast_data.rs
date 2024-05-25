use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct MulticastData {
    pub feed: String,
    pub topic: String,
    pub content_type: String,
    pub data_packets: Vec<DataPacket>
}

impl MulticastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::MulticastData
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: R) -> io::Result<MulticastData> {
        Ok(MulticastData {
            feed: String::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
            content_type: String::read(&mut reader).await?,
            data_packets: Vec::<DataPacket>::read(&mut reader).await?,
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.feed).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        (&self.content_type).write(&mut writer).await?;
        (&self.data_packets).write(&mut writer).await?;
        Ok(())
    }
}
