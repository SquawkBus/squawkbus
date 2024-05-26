use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use crate::io::Serializable;

use super::data_packet::DataPacket;
use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct MulticastData {
    pub topic: String,
    pub content_type: String,
    pub data_packets: Vec<DataPacket>
}

impl MulticastData {
    pub fn message_type(&self) -> MessageType {
        MessageType::MulticastData
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: &mut R) -> io::Result<MulticastData> {
        Ok(MulticastData {
            topic: String::read(&mut reader).await?,
            content_type: String::read(&mut reader).await?,
            data_packets: Vec::<DataPacket>::read(&mut reader).await?,
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        (&self.topic).write(&mut writer).await?;
        (&self.content_type).write(&mut writer).await?;
        (&self.data_packets).write(&mut writer).await?;
        Ok(())
    }
}
