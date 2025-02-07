use std::io::{self, Cursor};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct NotificationRequest {
    pub pattern: String,
    pub is_add: bool,
}

impl NotificationRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::NotificationRequest
    }
}

impl Serializable for NotificationRequest {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(NotificationRequest {
            pattern: String::deserialize(reader)?,
            is_add: bool::deserialize(reader)?,
        })
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.pattern.serialize(writer)?;
        self.is_add.serialize(writer)?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.pattern.size() + self.is_add.size()
    }
}
