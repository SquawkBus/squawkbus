use std::io::prelude::*;
use std::io;

use uuid::Uuid;

use crate::io::Serializable;

use super::message_type::MessageType;

// use crate::message_type::MessageType;

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

    pub fn read<R: Read>(mut reader: R) -> io::Result<AuthorizationRequest> {
        Ok(AuthorizationRequest {
            client_id: Uuid::read(&mut reader)?,
            host: String::read(&mut reader)?,
            user: String::read(&mut reader)?,
            feed: String::read(&mut reader)?,
            topic: String::read(&mut reader)?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.client_id).write(&mut writer)?;
        (&self.host).write(&mut writer)?;
        (&self.user).write(&mut writer)?;
        (&self.feed).write(&mut writer)?;
        (&self.topic).write(&mut writer)?;
        Ok(())
    }
}
