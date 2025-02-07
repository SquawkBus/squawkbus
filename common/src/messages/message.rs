use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::frame::{FrameReader, FrameWriter};
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

    pub fn deserialize(reader: &mut FrameReader) -> io::Result<Message> {
        match MessageType::deserialize(reader) {
            Ok(MessageType::AuthenticationRequest) => match AuthenticationRequest::read(reader) {
                Ok(message) => Ok(Message::AuthenticationRequest(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::AuthenticationResponse) => match AuthenticationResponse::read(reader) {
                Ok(message) => Ok(Message::AuthenticationResponse(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::ForwardedMulticastData) => match ForwardedMulticastData::read(reader) {
                Ok(message) => Ok(Message::ForwardedMulticastData(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::ForwardedSubscriptionRequest) => {
                match ForwardedSubscriptionRequest::read(reader) {
                    Ok(message) => Ok(Message::ForwardedSubscriptionRequest(message)),
                    Err(error) => Err(error),
                }
            }
            Ok(MessageType::ForwardedUnicastData) => match ForwardedUnicastData::read(reader) {
                Ok(message) => Ok(Message::ForwardedUnicastData(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::MulticastData) => match MulticastData::read(reader) {
                Ok(message) => Ok(Message::MulticastData(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::NotificationRequest) => match NotificationRequest::read(reader) {
                Ok(message) => Ok(Message::NotificationRequest(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::SubscriptionRequest) => match SubscriptionRequest::read(reader) {
                Ok(message) => Ok(Message::SubscriptionRequest(message)),
                Err(error) => Err(error),
            },
            Ok(MessageType::UnicastData) => match UnicastData::read(reader) {
                Ok(message) => Ok(Message::UnicastData(message)),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        }
    }

    pub fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        self.message_type().serialize(writer)?;
        match self {
            Message::AuthenticationRequest(message) => message.write(writer),
            Message::AuthenticationResponse(message) => message.write(writer),
            Message::ForwardedMulticastData(message) => message.write(writer),
            Message::ForwardedSubscriptionRequest(message) => message.write(writer),
            Message::ForwardedUnicastData(message) => message.write(writer),
            Message::MulticastData(message) => message.write(writer),
            Message::NotificationRequest(message) => message.write(writer),
            Message::SubscriptionRequest(message) => message.write(writer),
            Message::UnicastData(message) => message.write(writer),
        }
    }

    pub async fn read<R>(reader: &mut R) -> io::Result<Message>
    where
        R: AsyncReadExt + Unpin,
    {
        let mut frame = FrameReader::read(reader).await?;
        Message::deserialize(&mut frame)
    }

    pub async fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        let mut frame = FrameWriter::new();
        self.serialize(&mut frame)?;
        frame.write(writer).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test_message {
    use super::super::data_packet::DataPacket;

    use super::*;

    #[test]
    fn should_round_trip_authentication_request() {
        let initial = Message::AuthenticationRequest(AuthenticationRequest {
            method: "basic".into(),
            credentials: "mary".into(),
        });

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).expect("should deserialize");
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_authentication_response() {
        let initial = Message::AuthenticationResponse(AuthenticationResponse {
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
        });

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).expect("should deserialize");
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

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).unwrap();
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

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).unwrap();
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

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).unwrap();
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

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_notification_request() {
        let initial = Message::NotificationRequest(NotificationRequest {
            pattern: ".* LSE".into(),
            is_add: true,
        });

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_subscription_request() {
        let initial = Message::SubscriptionRequest(SubscriptionRequest {
            topic: "VOD LSE".into(),
            is_add: true,
        });

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).unwrap();
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

        let mut writer = FrameWriter::new();
        initial.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        let round_trip = Message::deserialize(&mut reader).unwrap();
        assert_eq!(initial, round_trip);
    }
}
