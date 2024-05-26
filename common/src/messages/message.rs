use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use crate::io::Serializable;

use super::message_type::MessageType;

use super::authorization_request::AuthorizationRequest;
use super::authorization_response::AuthorizationResponse;
use super::forwarded_multicast_data::ForwardedMulticastData;
use super::forwarded_subscription_request::ForwardedSubscriptionRequest;
use super::forwarded_unicast_data::ForwardedUnicastData;
use super::multicast_data::MulticastData;
use super::notification_request::NotificationRequest;
use super::subscription_request::SubscriptionRequest;
use super::unicast_data::UnicastData;

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    AuthorizationRequest(AuthorizationRequest),
    AuthorizationResponse(AuthorizationResponse),
    ForwardedMulticastData(ForwardedMulticastData),
    ForwardedSubscriptionRequest(ForwardedSubscriptionRequest),
    ForwardedUnicastData(ForwardedUnicastData),
    MulticastData(MulticastData),
    NotificationRequest(NotificationRequest),
    SubscriptionRequest(SubscriptionRequest),
    UnicastData(UnicastData),
}

impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::AuthorizationRequest(message) => message.message_type(),
            Message::AuthorizationResponse(message) => message.message_type(),
            Message::ForwardedMulticastData(message) => message.message_type(),
            Message::ForwardedSubscriptionRequest(message) => message.message_type(),
            Message::ForwardedUnicastData(message) => message.message_type(),
            Message::MulticastData(message) => message.message_type(),
            Message::NotificationRequest(message) => message.message_type(),
            Message::SubscriptionRequest(message) => message.message_type(),
            Message::UnicastData(message) => message.message_type(),
        }
    }

    pub async fn read<R: AsyncReadExt + Unpin>(mut reader: &mut R) -> io::Result<Message> {
        match MessageType::read(&mut reader).await {
            Ok(MessageType::AuthorizationRequest) => {
                match AuthorizationRequest::read(&mut reader).await {
                    Ok(message) => Ok(Message::AuthorizationRequest(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::AuthorizationResponse) => {
                match AuthorizationResponse::read(&mut reader).await {
                    Ok(message) => Ok(Message::AuthorizationResponse(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::ForwardedMulticastData) => {
                match ForwardedMulticastData::read(&mut reader).await {
                    Ok(message) => Ok(Message::ForwardedMulticastData(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::ForwardedSubscriptionRequest) => {
                match ForwardedSubscriptionRequest::read(&mut reader).await {
                    Ok(message) => Ok(Message::ForwardedSubscriptionRequest(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::ForwardedUnicastData) => {
                match ForwardedUnicastData::read(&mut reader).await {
                    Ok(message) => Ok(Message::ForwardedUnicastData(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::MulticastData) => match MulticastData::read(&mut reader).await {
                Ok(message) => Ok(Message::MulticastData(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::NotificationRequest) => match NotificationRequest::read(&mut reader).await {
                Ok(message) => Ok(Message::NotificationRequest(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::SubscriptionRequest) => match SubscriptionRequest::read(&mut reader).await {
                Ok(message) => Ok(Message::SubscriptionRequest(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::UnicastData) => match UnicastData::read(&mut reader).await {
                Ok(message) => Ok(Message::UnicastData(message)),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        }
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        self.message_type().write(&mut writer).await?;
        match self {
            Message::AuthorizationRequest(message) => message.write(&mut writer).await,
            Message::AuthorizationResponse(message) => message.write(&mut writer).await,
            Message::ForwardedMulticastData(message) => message.write(&mut writer).await,
            Message::ForwardedSubscriptionRequest(message) => message.write(&mut writer).await,
            Message::ForwardedUnicastData(message) => message.write(&mut writer).await,
            Message::MulticastData(message) => message.write(&mut writer).await,
            Message::NotificationRequest(message) => message.write(&mut writer).await,
            Message::SubscriptionRequest(message) => message.write(&mut writer).await,
            Message::UnicastData(message) => message.write(&mut writer).await,
        }
    }
}

#[cfg(test)]
mod test_message {
    use uuid::Uuid;

    use super::super::data_packet::DataPacket;

    use super::*;
    use std::{
        collections::HashSet,
        io::{Cursor, Seek},
    };

    #[tokio::test]
    async fn should_round_trip_authorization_request() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::AuthorizationRequest(AuthorizationRequest {
            host: String::from("host1"),
            user: String::from("mary"),
            client_id: Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap(),
            topic: String::from("VOD LSE"),
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_authorization_response() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::AuthorizationResponse(AuthorizationResponse {
            client_id: Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap(),
            topic: String::from("VOD LSE"),
            is_authorization_required: true,
            entitlements: HashSet::from([1, 2, 3]),
        });
        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_forwarded_multicast_data() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::ForwardedMulticastData(ForwardedMulticastData {
            host: String::from("host1"),
            user: String::from("mary"),
            topic: String::from("VOD LSE"),
            content_type: String::from("application/json"),
            data_packets: vec![DataPacket {
                entitlements: HashSet::from([-5i32, 1, 17]),
                data: vec![1u8, 2, 3, 4],
            }],
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_forwarded_subscription_request() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::ForwardedSubscriptionRequest(ForwardedSubscriptionRequest {
            host: String::from("host1"),
            user: String::from("mary"),
            client_id: Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap(),
            topic: String::from("VOD LSE"),
            is_add: true,
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_forwarded_unicast_data() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::ForwardedUnicastData(ForwardedUnicastData {
            host: String::from("host1"),
            user: String::from("mary"),
            client_id: Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap(),
            topic: String::from("VOD LSE"),
            content_type: String::from("application/json"),
            data_packets: vec![DataPacket {
                entitlements: HashSet::from([-5i32, 1, 17]),
                data: vec![1u8, 2, 3, 4],
            }],
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_multicast_data() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::MulticastData(MulticastData {
            topic: String::from("VOD LSE"),
            content_type: String::from("application/json"),
            data_packets: vec![DataPacket {
                entitlements: HashSet::from([-5i32, 1, 17]),
                data: vec![1u8, 2, 3, 4],
            }],
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_notification_request() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::NotificationRequest(NotificationRequest {
            pattern: String::from(".* LSE"),
            is_add: true,
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_subscription_request() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::SubscriptionRequest(SubscriptionRequest {
            topic: String::from("VOD LSE"),
            is_add: true,
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }

    #[tokio::test]
    async fn should_roundtrip_unicast_data() {
        let mut cursor = Cursor::new(Vec::new());

        let initial = Message::UnicastData(UnicastData {
            client_id: Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap(),
            topic: String::from("VOD LSE"),
            content_type: String::from("application/json"),
            data_packets: vec![DataPacket {
                entitlements: HashSet::from([-5i32, 1, 17]),
                data: vec![1u8, 2, 3, 4],
            }],
        });

        initial.write(&mut cursor).await.expect("should serialize");

        cursor.rewind().unwrap();
        let round_trip = Message::read(&mut cursor).await.unwrap();
        assert_eq!(initial, round_trip);
    }
}
