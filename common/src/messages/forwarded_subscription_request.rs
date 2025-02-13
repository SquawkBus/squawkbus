use std::io::{self, Cursor};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct ForwardedSubscriptionRequest {
    pub host: String,
    pub user: String,
    pub client_id: String,
    pub topic: String,
    pub is_add: bool,
}

impl ForwardedSubscriptionRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::ForwardedSubscriptionRequest
    }
}

impl Serializable for ForwardedSubscriptionRequest {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(ForwardedSubscriptionRequest {
            host: String::deserialize(reader)?,
            user: String::deserialize(reader)?,
            client_id: String::deserialize(reader)?,
            topic: String::deserialize(reader)?,
            is_add: bool::deserialize(reader)?,
        })
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.host.serialize(writer)?;
        self.user.serialize(writer)?;
        self.client_id.serialize(writer)?;
        self.topic.serialize(writer)?;
        self.is_add.serialize(writer)?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.host.size()
            + self.user.size()
            + self.client_id.size()
            + self.topic.size()
            + self.is_add.size()
    }
}
