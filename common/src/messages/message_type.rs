use std::io::{self, Cursor, ErrorKind};

use crate::io::Serializable;

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum MessageType {
    AuthenticationRequest = 1,
    AuthenticationResponse = 2,
    MulticastData = 3,
    UnicastData = 4,
    ForwardedSubscriptionRequest = 5,
    NotificationRequest = 6,
    SubscriptionRequest = 7,
    ForwardedMulticastData = 8,
    ForwardedUnicastData = 9,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MessageType::AuthenticationRequest),
            2 => Ok(MessageType::AuthenticationResponse),
            3 => Ok(MessageType::MulticastData),
            4 => Ok(MessageType::UnicastData),
            5 => Ok(MessageType::ForwardedSubscriptionRequest),
            6 => Ok(MessageType::NotificationRequest),
            7 => Ok(MessageType::SubscriptionRequest),
            8 => Ok(MessageType::ForwardedMulticastData),
            9 => Ok(MessageType::ForwardedUnicastData),
            _ => Err(()),
        }
    }
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::AuthenticationRequest => 1,
            MessageType::AuthenticationResponse => 2,
            MessageType::MulticastData => 3,
            MessageType::UnicastData => 4,
            MessageType::ForwardedSubscriptionRequest => 5,
            MessageType::NotificationRequest => 6,
            MessageType::SubscriptionRequest => 7,
            MessageType::ForwardedMulticastData => 8,
            MessageType::ForwardedUnicastData => 9,
        }
    }
}

impl Serializable for MessageType {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        let byte: u8 = (*self).into();
        byte.serialize(writer)?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<MessageType> {
        let byte = u8::deserialize(reader)?;
        MessageType::try_from(byte).map_err(|_| io::Error::new(ErrorKind::Other, "invalid"))
    }

    fn size(&self) -> usize {
        1
    }
}
