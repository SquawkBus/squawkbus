use std::io;

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct AuthorizationResponse {
    pub client_id: String,
    pub topic: String,
    pub is_authorization_required: bool,
    pub entitlement: i32,
}

impl AuthorizationResponse {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthorizationResponse
    }

    pub fn read(reader: &mut FrameReader) -> io::Result<AuthorizationResponse> {
        Ok(AuthorizationResponse {
            client_id: String::read(reader)?,
            topic: String::read(reader)?,
            is_authorization_required: bool::read(reader)?,
            entitlement: i32::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        (&self.client_id).write(writer)?;
        (&self.topic).write(writer)?;
        (&self.is_authorization_required).write(writer)?;
        (&self.entitlement).write(writer)?;
        Ok(())
    }
}
