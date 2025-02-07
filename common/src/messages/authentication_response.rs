use std::io;

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct AuthenticationResponse {
    pub client_id: String,
}

impl AuthenticationResponse {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthenticationResponse
    }

    pub fn read(reader: &mut FrameReader) -> io::Result<AuthenticationResponse> {
        Ok(AuthenticationResponse {
            client_id: String::deserialize(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.client_id).serialize(writer)?;
        Ok(())
    }
}
