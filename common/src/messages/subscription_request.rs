use std::io::{self, Cursor};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct SubscriptionRequest {
    pub topic: String,
    pub is_add: bool,
}

impl SubscriptionRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::SubscriptionRequest
    }
}

impl Serializable for SubscriptionRequest {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(SubscriptionRequest {
            topic: String::deserialize(reader)?,
            is_add: bool::deserialize(reader)?,
        })
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.topic.serialize(writer)?;
        self.is_add.serialize(writer)?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.topic.size() + self.is_add.size()
    }
}
