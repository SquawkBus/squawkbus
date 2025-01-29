use std::io;

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

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

    pub fn read(reader: &mut FrameReader) -> io::Result<SubscriptionRequest> {
        Ok(SubscriptionRequest {
            topic: String::read(reader)?,
            is_add: bool::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.topic).write(writer)?;
        self.is_add.write(writer)?;
        Ok(())
    }
}
