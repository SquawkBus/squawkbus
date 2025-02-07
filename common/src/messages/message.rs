use std::io::Cursor;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::io::Serializable;

use super::message_type::MessageType;

use super::authentication_request::AuthenticationRequest;
use super::authentication_response::AuthenticationResponse;
use super::forwarded_multicast_data::ForwardedMulticastData;
use super::forwarded_subscription_request::ForwardedSubscriptionRequest;
use super::forwarded_unicast_data::ForwardedUnicastData;
use super::multicast_data::MulticastData;
use super::notification_request::NotificationRequest;
use super::subscription_request::SubscriptionRequest;
use super::unicast_data::UnicastData;

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    AuthenticationRequest(AuthenticationRequest),
    AuthenticationResponse(AuthenticationResponse),
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
            Message::AuthenticationRequest(message) => message.message_type(),
            Message::AuthenticationResponse(message) => message.message_type(),
            Message::ForwardedMulticastData(message) => message.message_type(),
            Message::ForwardedSubscriptionRequest(message) => message.message_type(),
            Message::ForwardedUnicastData(message) => message.message_type(),
            Message::MulticastData(message) => message.message_type(),
            Message::NotificationRequest(message) => message.message_type(),
            Message::SubscriptionRequest(message) => message.message_type(),
            Message::UnicastData(message) => message.message_type(),
        }
    }

    pub async fn read<R>(reader: &mut R) -> io::Result<Message>
    where
        R: AsyncReadExt + Unpin,
    {
        let mut len_buf = [0_u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);
        let mut buf: Vec<u8> = vec![0; len as usize];
        reader.read_exact(&mut buf).await?;
        let mut cursor = Cursor::new(buf);
        Message::deserialize(&mut cursor)
    }

    pub async fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        let bytes_to_write = self.size();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(4 + bytes_to_write));

        (bytes_to_write as u32).serialize(&mut cursor)?;
        self.serialize(&mut cursor)?;

        writer.write_all(cursor.get_ref().as_slice()).await
    }
}

impl Serializable for Message {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Message> {
        match MessageType::deserialize(reader) {
            Ok(MessageType::AuthenticationRequest) => {
                match AuthenticationRequest::deserialize(reader) {
                    Ok(message) => Ok(Message::AuthenticationRequest(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::AuthenticationResponse) => {
                match AuthenticationResponse::deserialize(reader) {
                    Ok(message) => Ok(Message::AuthenticationResponse(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::ForwardedMulticastData) => {
                match ForwardedMulticastData::deserialize(reader) {
                    Ok(message) => Ok(Message::ForwardedMulticastData(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::ForwardedSubscriptionRequest) => {
                match ForwardedSubscriptionRequest::deserialize(reader) {
                    Ok(message) => Ok(Message::ForwardedSubscriptionRequest(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::ForwardedUnicastData) => {
                match ForwardedUnicastData::deserialize(reader) {
                    Ok(message) => Ok(Message::ForwardedUnicastData(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::MulticastData) => match MulticastData::deserialize(reader) {
                Ok(message) => Ok(Message::MulticastData(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::NotificationRequest) => {
                match NotificationRequest::deserialize(reader) {
                    Ok(message) => Ok(Message::NotificationRequest(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::SubscriptionRequest) => {
                match SubscriptionRequest::deserialize(reader) {
                    Ok(message) => Ok(Message::SubscriptionRequest(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::UnicastData) => match UnicastData::deserialize(reader) {
                Ok(message) => Ok(Message::UnicastData(message)),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        }
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.message_type().serialize(writer)?;
        match self {
            Message::AuthenticationRequest(message) => message.serialize(writer),
            Message::AuthenticationResponse(message) => message.serialize(writer),
            Message::ForwardedMulticastData(message) => message.serialize(writer),
            Message::ForwardedSubscriptionRequest(message) => message.serialize(writer),
            Message::ForwardedUnicastData(message) => message.serialize(writer),
            Message::MulticastData(message) => message.serialize(writer),
            Message::NotificationRequest(message) => message.serialize(writer),
            Message::SubscriptionRequest(message) => message.serialize(writer),
            Message::UnicastData(message) => message.serialize(writer),
        }
    }

    fn size(&self) -> usize {
        self.message_type().size()
            + match self {
                Message::AuthenticationRequest(message) => message.size(),
                Message::AuthenticationResponse(message) => message.size(),
                Message::ForwardedMulticastData(message) => message.size(),
                Message::ForwardedSubscriptionRequest(message) => message.size(),
                Message::ForwardedUnicastData(message) => message.size(),
                Message::MulticastData(message) => message.size(),
                Message::NotificationRequest(message) => message.size(),
                Message::SubscriptionRequest(message) => message.size(),
                Message::UnicastData(message) => message.size(),
            }
    }
}

#[cfg(test)]
mod test_message {
    use super::super::data_packet::DataPacket;
    use super::*;
    use std::io::Seek;

    #[test]
    fn should_round_trip_authentication_request() {
        let initial = Message::AuthenticationRequest(AuthenticationRequest {
            method: "basic".into(),
            credentials: "mary".into(),
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).expect("should deserialize");
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_authentication_response() {
        let initial = Message::AuthenticationResponse(AuthenticationResponse {
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).expect("should deserialize");
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_multicast_data() {
        let initial = Message::ForwardedMulticastData(ForwardedMulticastData {
            host: "host1".into(),
            user: "mary".into(),
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_subscription_request() {
        let initial = Message::ForwardedSubscriptionRequest(ForwardedSubscriptionRequest {
            host: "host1".into(),
            user: "mary".into(),
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            topic: "VOD LSE".into(),
            is_add: true,
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_unicast_data() {
        let initial = Message::ForwardedUnicastData(ForwardedUnicastData {
            host: "host1".into(),
            user: "mary".into(),
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_multicast_data() {
        let initial = Message::MulticastData(MulticastData {
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_notification_request() {
        let initial = Message::NotificationRequest(NotificationRequest {
            pattern: ".* LSE".into(),
            is_add: true,
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_subscription_request() {
        let initial = Message::SubscriptionRequest(SubscriptionRequest {
            topic: "VOD LSE".into(),
            is_add: true,
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_unicast_data() {
        let initial = Message::UnicastData(UnicastData {
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        });

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }
}
