use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use uuid::Uuid;

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct ForwardedSubscriptionRequest {
    pub host: String,
    pub user: String,
    pub client_id: Uuid,
    pub topic: String,
    pub is_add: bool,
}

impl ForwardedSubscriptionRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedSubscriptionRequest
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: R) -> io::Result<ForwardedSubscriptionRequest> {
        Ok(ForwardedSubscriptionRequest {
            host: String::read(&mut reader).await?,
            user: String::read(&mut reader).await?,
            client_id: Uuid::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
            is_add: bool::read(&mut reader).await?
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.host).write(&mut writer).await?;
        (&self.user).write(&mut writer).await?;
        (&self.client_id).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        self.is_add.write(&mut writer).await?;
        Ok(())
    }
}
