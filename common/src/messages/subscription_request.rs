use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct SubscriptionRequest {
    pub topic: String,
    pub is_add: bool,
}

impl SubscriptionRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::SubscriptionRequest
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: &mut R) -> io::Result<SubscriptionRequest> {
        Ok(SubscriptionRequest {
            topic: String::read(&mut reader).await?,
            is_add: bool::read(&mut reader).await?
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        (&self.topic).write(&mut writer).await?;
        self.is_add.write(&mut writer).await?;
        Ok(())
    }
}
