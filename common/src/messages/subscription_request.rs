use std::io::prelude::*;
use std::io;

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

    pub fn read<R: Read>(mut reader: R) -> io::Result<SubscriptionRequest> {
        Ok(SubscriptionRequest {
            feed: String::read(&mut reader)?,
            topic: String::read(&mut reader)?,
            is_add: bool::read(&mut reader)?
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.feed).write(&mut writer)?;
        (&self.topic).write(&mut writer)?;
        self.is_add.write(&mut writer)?;
        Ok(())
    }
}
