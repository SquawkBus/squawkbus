use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct NotificationRequest {
    pub feed: String,
    pub is_add: bool,
}

impl NotificationRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::NotificationRequest
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: R) -> io::Result<NotificationRequest> {
        Ok(NotificationRequest {
            feed: String::read(&mut reader).await?,
            is_add: bool::read(&mut reader).await?
        })
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.feed).write(&mut writer).await?;
        self.is_add.write(&mut writer).await?;
        Ok(())
    }
}
