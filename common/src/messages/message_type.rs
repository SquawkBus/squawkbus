use std::io::{self, ErrorKind};

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum MessageType {
    MulticastData = 1,
    UnicastData = 2,
    ForwardedSubscriptionRequest = 3,
    NotificationRequest = 4,
    SubscriptionRequest = 5,
    AuthorizationRequest = 6,
    AuthorizationResponse = 7,
    ForwardedMulticastData = 8,
    ForwardedUnicastData = 9,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MessageType::MulticastData),
            2 => Ok(MessageType::UnicastData),
            3 => Ok(MessageType::ForwardedSubscriptionRequest),
            4 => Ok(MessageType::NotificationRequest),
            5 => Ok(MessageType::SubscriptionRequest),
            6 => Ok(MessageType::AuthorizationRequest),
            7 => Ok(MessageType::AuthorizationResponse),
            8 => Ok(MessageType::ForwardedMulticastData),
            9 => Ok(MessageType::ForwardedUnicastData),
            _ => Err(()),
        }
    }
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::MulticastData => 1,
            MessageType::UnicastData => 2,
            MessageType::ForwardedSubscriptionRequest => 3,
            MessageType::NotificationRequest => 4,
            MessageType::SubscriptionRequest => 5,
            MessageType::AuthorizationRequest => 6,
            MessageType::AuthorizationResponse => 7,
            MessageType::ForwardedMulticastData => 8,
            MessageType::ForwardedUnicastData => 9,
        }
    }
}

impl Serializable for MessageType {
    fn write(&self, writer: &mut FrameWriter) -> io::Result<()> {
        let byte: u8 = (*self).into();
        byte.write(writer)?;
        Ok(())
    }

    fn read(reader: &mut FrameReader) -> io::Result<MessageType> {
        let byte = u8::read(reader)?;
        MessageType::try_from(byte).map_err(|_| io::Error::new(ErrorKind::Other, "invalid"))
    }
}
