use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use uuid::Uuid;

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct AuthorizationRequest {
    pub host: String,
    pub user: String,
    pub client_id: Uuid,
    pub topic: String
}

impl AuthorizationRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthorizationRequest
    }

    pub async fn read<R: AsyncReadExt+Unpin>(mut reader: R) -> io::Result<AuthorizationRequest> {
        Ok(AuthorizationRequest {
            host: String::read(&mut reader).await?,
            user: String::read(&mut reader).await?,
            client_id: Uuid::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
        })
    }

    pub async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.host).write(&mut writer).await?;
        (&self.user).write(&mut writer).await?;
        (&self.client_id).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        Ok(())
    }
}
