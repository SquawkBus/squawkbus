use std::collections::HashSet;
use std::io::prelude::*;
use std::io;

use uuid::Uuid;

use crate::io::Serializable;

use super::message_type::MessageType;

#[derive(PartialEq, Debug)]
pub struct AuthorizationResponse {
    pub client_id: Uuid,
    pub feed: String,
    pub topic: String,
    pub is_authorization_required: bool,
    pub entitlements: HashSet<i32>,
}

impl AuthorizationResponse {
    pub fn message_type(&self) -> MessageType {
        MessageType::AuthorizationResponse
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<AuthorizationResponse> {
        Ok(AuthorizationResponse {
            client_id: Uuid::read(&mut reader)?,
            feed: String::read(&mut reader)?,
            topic: String::read(&mut reader)?,
            is_authorization_required: bool::read(&mut reader)?,
            entitlements: HashSet::<i32>::read(&mut reader)?,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (&self.client_id).write(&mut writer)?;
        (&self.feed).write(&mut writer)?;
        (&self.topic).write(&mut writer)?;
        (&self.is_authorization_required).write(&mut writer)?;
        (&self.entitlements).write(&mut writer)?;
        Ok(())
    }
}
