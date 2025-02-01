use std::io;

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct AuthenticationRequest {
    pub method: String,
    pub data: String,
}

impl AuthenticationRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthenticationRequest
    }

    pub fn read(reader: &mut FrameReader) -> io::Result<AuthenticationRequest> {
        Ok(AuthenticationRequest {
            method: String::deserialize(reader)?,
            data: String::deserialize(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.method).serialize(writer)?;
        (&self.data).serialize(writer)?;
        Ok(())
    }
}
