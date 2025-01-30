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
            host: String::deserialize(reader)?,
            user: String::deserialize(reader)?,
            client_id: String::deserialize(reader)?,
            topic: String::deserialize(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.host).serialize(writer)?;
        (&self.user).serialize(writer)?;
        (&self.client_id).serialize(writer)?;
        (&self.topic).serialize(writer)?;
        Ok(())
    }
}
