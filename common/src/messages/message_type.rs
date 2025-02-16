use std::io::{self, Cursor, ErrorKind};

use crate::io::Serializable;

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum MessageType {
    AUTHENTICATION_REQUEST = 1,
    AUTHENTICATION_RESPONSE = 2,
    MULTICAST_DATA = 3,
    UNICAST_DATA = 4,
    FORWARDED_SUBSCRIPTION_REQUEST = 5,
    NOTIFICATION_REQUEST = 6,
    SUBSCRIPTION_REQUEST = 7,
    FORWARDED_MULTICAST_DATA = 8,
    FORWARDED_UNICAST_DATA = 9,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MessageType::AUTHENTICATION_REQUEST),
            2 => Ok(MessageType::AUTHENTICATION_RESPONSE),
            3 => Ok(MessageType::MULTICAST_DATA),
            4 => Ok(MessageType::UNICAST_DATA),
            5 => Ok(MessageType::FORWARDED_SUBSCRIPTION_REQUEST),
            6 => Ok(MessageType::NOTIFICATION_REQUEST),
            7 => Ok(MessageType::SUBSCRIPTION_REQUEST),
            8 => Ok(MessageType::FORWARDED_MULTICAST_DATA),
            9 => Ok(MessageType::FORWARDED_UNICAST_DATA),
            _ => Err(()),
        }
    }
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::AUTHENTICATION_REQUEST => 1,
            MessageType::AUTHENTICATION_RESPONSE => 2,
            MessageType::MULTICAST_DATA => 3,
            MessageType::UNICAST_DATA => 4,
            MessageType::FORWARDED_SUBSCRIPTION_REQUEST => 5,
            MessageType::NOTIFICATION_REQUEST => 6,
            MessageType::SUBSCRIPTION_REQUEST => 7,
            MessageType::FORWARDED_MULTICAST_DATA => 8,
            MessageType::FORWARDED_UNICAST_DATA => 9,
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
