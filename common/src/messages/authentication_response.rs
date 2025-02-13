use std::io::{self, Cursor};

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(Debug, PartialEq, Clone)]
pub struct AuthenticationResponse {
    pub client_id: String,
}

impl AuthenticationResponse {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthenticationResponse
    }
}

impl Serializable for AuthenticationResponse {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.client_id.serialize(writer)?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<AuthenticationResponse> {
        Ok(AuthenticationResponse {
            client_id: String::deserialize(reader)?,
        })
    }

    fn size(&self) -> usize {
        self.client_id.size()
    }
}
