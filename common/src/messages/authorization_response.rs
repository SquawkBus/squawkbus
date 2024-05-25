use std::collections::HashSet;

use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use uuid::Uuid;

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct AuthorizationResponse {
    pub client_id: Uuid,
    pub feed: String,
    pub topic: String,
    pub is_authorization_required: bool,
    pub entitlements: HashSet<i32>,
}

impl AuthorizationResponse {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthorizationResponse
    }

    pub async fn read<R: AsyncReadExt+Unpin>(mut reader: R) -> io::Result<AuthorizationResponse> {
        Ok(AuthorizationResponse {
            client_id: Uuid::read(&mut reader).await?,
            feed: String::read(&mut reader).await?,
            topic: String::read(&mut reader).await?,
            is_authorization_required: bool::read(&mut reader).await?,
            entitlements: HashSet::<i32>::read(&mut reader).await?,
        })
    }

    pub async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: W) -> io::Result<()> {
        (&self.client_id).write(&mut writer).await?;
        (&self.feed).write(&mut writer).await?;
        (&self.topic).write(&mut writer).await?;
        (&self.is_authorization_required).write(&mut writer).await?;
        (&self.entitlements).write(&mut writer).await?;
        Ok(())
    }
}
