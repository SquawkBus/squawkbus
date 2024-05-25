use std::io::prelude::*;
use std::io;

use uuid::Uuid;

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct ForwardedSubscriptionRequest {
    pub host: String,
    pub user: String,
    pub client_id: Uuid,
    pub feed: String,
    pub topic: String,
    pub is_add: bool,
}

impl ForwardedSubscriptionRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedSubscriptionRequest
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<ForwardedSubscriptionRequest> {
        Ok(ForwardedSubscriptionRequest {
            host: String::read(&mut reader)?,
            user: String::read(&mut reader)?,
            client_id: Uuid::read(&mut reader)?,
            feed: String::read(&mut reader)?,
            topic: String::read(&mut reader)?,
            is_add: bool::read(&mut reader)?
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.host).write(&mut writer)?;
        (&self.user).write(&mut writer)?;
        (&self.client_id).write(&mut writer)?;
        (&self.feed).write(&mut writer)?;
        (&self.topic).write(&mut writer)?;
        self.is_add.write(&mut writer)?;
        Ok(())
    }
}
