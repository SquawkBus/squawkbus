use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct SubscriptionRequest {
    pub feed: String,
    pub topic: String,
    pub is_add: bool,
}

impl SubscriptionRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::SubscriptionRequest
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: R) -> io::Result<SubscriptionRequest> {
        Ok(SubscriptionRequest {
            feed: String::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
            is_add: bool::read(&mut reader).await?
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.feed).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        self.is_add.write(&mut writer).await?;
        Ok(())
    }
}
