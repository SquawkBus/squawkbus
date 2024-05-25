use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

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

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: R) -> io::Result<ForwardedUnicastData> {
        Ok(ForwardedUnicastData {
            host: String::read(&mut reader).await?,
            user: String::read(&mut reader).await?,
            client_id: Uuid::read(&mut reader).await?,
            feed: String::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
            content_type: String::read(&mut reader).await?,
            data_packets: Vec::<DataPacket>::read(&mut reader).await?,
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.host).write(&mut writer).await?;
        (&self.user).write(&mut writer).await?;
        (&self.client_id).write(&mut writer).await?;
        (&self.feed).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        (&self.content_type).write(&mut writer).await?;
        (&self.data_packets).write(&mut writer).await?;
        Ok(())
    }
}
