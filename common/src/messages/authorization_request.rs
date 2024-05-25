use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use uuid::Uuid;

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct AuthorizationRequest {
    pub client_id: Uuid,
    pub host: String,
    pub user: String,
    pub feed: String,
    pub topic: String
}

impl AuthorizationRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthorizationRequest
    }

    pub async fn read<R: AsyncReadExt+Unpin>(mut reader: R) -> io::Result<AuthorizationRequest> {
        Ok(AuthorizationRequest {
            client_id: Uuid::read(&mut reader).await?,
            host: String::read(&mut reader).await?,
            user: String::read(&mut reader).await?,
            feed: String::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
        })
    }

    pub async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.client_id).write(&mut writer).await?;
        (&self.host).write(&mut writer).await?;
        (&self.user).write(&mut writer).await?;
        (&self.feed).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        Ok(())
    }
}
