use std::io::{self, Cursor};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct AuthenticationRequest {
    pub method: String,
    pub credentials: Vec<u8>,
}

impl AuthenticationRequest {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthenticationRequest
    }
}

impl Serializable for AuthenticationRequest {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.method.serialize(writer)?;
        self.credentials.serialize(writer)?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        Ok(AuthenticationRequest {
            method: String::deserialize(reader)?,
            credentials: Vec::deserialize(reader)?,
        })
    }

    fn size(&self) -> usize {
        self.method.size() + self.credentials.size()
    }
}
