use std::io;

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct AuthorizationRequest {
    pub host: String,
    pub user: String,
    pub client_id: String,
    pub topic: String,
}

impl AuthorizationRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthorizationRequest
    }

    pub fn read(reader: &mut FrameReader) -> io::Result<AuthorizationRequest> {
        Ok(AuthorizationRequest {
            host: String::read(reader)?,
            user: String::read(reader)?,
            client_id: String::read(reader)?,
            topic: String::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.host).write(writer)?;
        (&self.user).write(writer)?;
        (&self.client_id).write(writer)?;
        (&self.topic).write(writer)?;
        Ok(())
    }
}
