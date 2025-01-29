use std::io;

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

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

    pub fn read(reader: &mut FrameReader) -> io::Result<NotificationRequest> {
        Ok(NotificationRequest {
            pattern: String::read(reader)?,
            is_add: bool::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.pattern).write(writer)?;
        self.is_add.write(writer)?;
        Ok(())
    }
}
