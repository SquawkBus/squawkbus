use tokio::io::{self,AsyncReadExt,AsyncWriteExt,ErrorKind};

use crate::io::Serializable;

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum MessageType
{
    MulticastData = 1,
    UnicastData = 2,
    ForwardedSubscriptionRequest = 3,
    NotificationRequest = 4,
    SubscriptionRequest = 5,
    AuthorizationRequest = 6,
    AuthorizationResponse = 7,
    ForwardedMulticastData = 8,
    ForwardedUnicastData = 9
}

impl MessageType {
    pub fn try_into(value: u8) -> Option<MessageType> {
        match value {
            1 => Some(MessageType::MulticastData),
            2 => Some(MessageType::UnicastData),
            3 => Some(MessageType::ForwardedSubscriptionRequest),
            4 => Some(MessageType::NotificationRequest),
            5 => Some(MessageType::SubscriptionRequest),
            6 => Some(MessageType::AuthorizationRequest),
            7 => Some(MessageType::AuthorizationResponse),
            8 => Some(MessageType::ForwardedMulticastData),
            9 => Some(MessageType::ForwardedUnicastData),
            _ => None
        }
    }
}

impl Serializable for MessageType {
    async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> io::Result<()> {
        (*self as u8).write(&mut writer).await?;
        Ok(())            
    }

    async fn read<R: AsyncReadExt + Unpin>(mut reader: R) -> io::Result<MessageType> {
        match u8::read(&mut reader).await {
            Ok(1) => Ok(MessageType::MulticastData),
            Ok(2) => Ok(MessageType::UnicastData),
            Ok(3) => Ok(MessageType::ForwardedSubscriptionRequest),
            Ok(4) => Ok(MessageType::NotificationRequest),
            Ok(5) => Ok(MessageType::SubscriptionRequest),
            Ok(6) => Ok(MessageType::AuthorizationRequest),
            Ok(7) => Ok(MessageType::AuthorizationResponse),
            Ok(8) => Ok(MessageType::ForwardedMulticastData),
            Ok(9) => Ok(MessageType::ForwardedUnicastData),
            _ => Err(io::Error::new(ErrorKind::Other, "invalid message type")),
        }
    }
}
