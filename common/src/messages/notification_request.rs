use std::io::prelude::*;
use std::io;

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

    pub fn read<R: Read>(mut reader: R) -> io::Result<NotificationRequest> {
        Ok(NotificationRequest {
            feed: String::read(&mut reader)?,
            is_add: bool::read(&mut reader)?
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.feed).write(&mut writer)?;
        self.is_add.write(&mut writer)?;
        Ok(())
    }
}
