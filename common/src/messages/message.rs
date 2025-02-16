use std::io::{self, Cursor};

use crate::io::Serializable;

use super::message_type::MessageType;

use super::DataPacket;

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    AuthenticationRequest(String, Vec<u8>),
    AuthenticationResponse(String),
    ForwardedMulticastData(String, String, String, Vec<DataPacket>),
    ForwardedSubscriptionRequest(String, String, String, String, bool),
    // host, user, client_id, topic, data_packets.
    ForwardedUnicastData(String, String, String, String, Vec<DataPacket>),
    // topic, data_packets
    MulticastData(String, Vec<DataPacket>),
    // pattern, is_add
    NotificationRequest(String, bool),
    // topic, is_add
    SubscriptionRequest(String, bool),
    // client_id, topic, data_packets.
    UnicastData(String, String, Vec<DataPacket>),
}

impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::AuthenticationRequest(_, _) => MessageType::AUTHENTICATION_REQUEST,
            Message::AuthenticationResponse(_) => MessageType::AUTHENTICATION_RESPONSE,
            Message::ForwardedMulticastData(_, _, _, _) => MessageType::FORWARDED_MULTICAST_DATA,
            Message::ForwardedSubscriptionRequest(_, _, _, _, _) => {
                MessageType::FORWARDED_SUBSCRIPTION_REQUEST
            }
            Message::ForwardedUnicastData(_, _, _, _, _) => MessageType::FORWARDED_UNICAST_DATA,
            Message::MulticastData(_, _) => MessageType::MULTICAST_DATA,
            Message::NotificationRequest(_, _) => MessageType::NOTIFICATION_REQUEST,
            Message::SubscriptionRequest(_, _) => MessageType::SUBSCRIPTION_REQUEST,
            Message::UnicastData(_, _, _) => MessageType::UNICAST_DATA,
        }
    }
}

impl Serializable for Message {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Message> {
        match MessageType::deserialize(reader) {
            Ok(MessageType::AUTHENTICATION_REQUEST) => Ok(Message::AuthenticationRequest(
                String::deserialize(reader)?,
                Vec::deserialize(reader)?,
            )),
            Ok(MessageType::AUTHENTICATION_RESPONSE) => Ok(Message::AuthenticationResponse(
                String::deserialize(reader)?,
            )),
            Ok(MessageType::FORWARDED_MULTICAST_DATA) => Ok(Message::ForwardedMulticastData(
                String::deserialize(reader)?,
                String::deserialize(reader)?,
                String::deserialize(reader)?,
                Vec::<DataPacket>::deserialize(reader)?,
            )),
            Ok(MessageType::FORWARDED_SUBSCRIPTION_REQUEST) => {
                Ok(Message::ForwardedSubscriptionRequest(
                    String::deserialize(reader)?,
                    String::deserialize(reader)?,
                    String::deserialize(reader)?,
                    String::deserialize(reader)?,
                    bool::deserialize(reader)?,
                ))
            }
            Ok(MessageType::FORWARDED_UNICAST_DATA) => Ok(Message::ForwardedUnicastData(
                String::deserialize(reader)?,
                String::deserialize(reader)?,
                String::deserialize(reader)?,
                String::deserialize(reader)?,
                Vec::<DataPacket>::deserialize(reader)?,
            )),
            Ok(MessageType::MULTICAST_DATA) => Ok(Message::MulticastData(
                String::deserialize(reader)?,
                Vec::<DataPacket>::deserialize(reader)?,
            )),
            Ok(MessageType::NOTIFICATION_REQUEST) => Ok(Message::NotificationRequest(
                String::deserialize(reader)?,
                bool::deserialize(reader)?,
            )),
            Ok(MessageType::SUBSCRIPTION_REQUEST) => Ok(Message::SubscriptionRequest(
                String::deserialize(reader)?,
                bool::deserialize(reader)?,
            )),
            Ok(MessageType::UNICAST_DATA) => Ok(Message::UnicastData(
                String::deserialize(reader)?,
                String::deserialize(reader)?,
                Vec::<DataPacket>::deserialize(reader)?,
            )),
            Err(error) => Err(error),
        }
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.message_type().serialize(writer)?;
        match self {
            Message::AuthenticationRequest(method, credentials) => {
                method.serialize(writer)?;
                credentials.serialize(writer)?;
                Ok(())
            }
            Message::AuthenticationResponse(client_id) => {
                client_id.serialize(writer)?;
                Ok(())
            }
            Message::ForwardedMulticastData(host, user, topic, data_packets) => {
                host.serialize(writer)?;
                user.serialize(writer)?;
                topic.serialize(writer)?;
                data_packets.serialize(writer)?;
                Ok(())
            }
            Message::ForwardedSubscriptionRequest(host, user, client_id, topic, is_add) => {
                host.serialize(writer)?;
                user.serialize(writer)?;
                client_id.serialize(writer)?;
                topic.serialize(writer)?;
                is_add.serialize(writer)?;
                Ok(())
            }
            Message::ForwardedUnicastData(host, user, client_id, topic, data_packets) => {
                host.serialize(writer)?;
                user.serialize(writer)?;
                client_id.serialize(writer)?;
                topic.serialize(writer)?;
                data_packets.serialize(writer)?;
                Ok(())
            }
            Message::MulticastData(topic, data_packets) => {
                topic.serialize(writer)?;
                data_packets.serialize(writer)?;
                Ok(())
            }
            Message::NotificationRequest(pattern, is_add) => {
                pattern.serialize(writer)?;
                is_add.serialize(writer)?;
                Ok(())
            }
            Message::SubscriptionRequest(topic, is_add) => {
                topic.serialize(writer)?;
                is_add.serialize(writer)?;
                Ok(())
            }
            Message::UnicastData(client_id, topic, data_packets) => {
                client_id.serialize(writer)?;
                topic.serialize(writer)?;
                data_packets.serialize(writer)?;
                Ok(())
            }
        }
    }

    fn size(&self) -> usize {
        self.message_type().size()
            + match self {
                Message::AuthenticationRequest(method, credentials) => {
                    method.size() + credentials.size()
                }
                Message::AuthenticationResponse(client_id) => client_id.size(),
                Message::ForwardedMulticastData(host, user, topic, data_packets) => {
                    host.size() + user.size() + topic.size() + data_packets.size()
                }
                Message::ForwardedSubscriptionRequest(host, user, client_id, topic, is_add) => {
                    host.size() + user.size() + client_id.size() + topic.size() + is_add.size()
                }
                Message::ForwardedUnicastData(host, user, client_id, topic, data_packets) => {
                    host.size()
                        + user.size()
                        + client_id.size()
                        + topic.size()
                        + data_packets.size()
                }
                Message::MulticastData(topic, data_packets) => topic.size() + data_packets.size(),
                Message::NotificationRequest(pattern, is_add) => pattern.size() + is_add.size(),
                Message::SubscriptionRequest(topic, is_add) => topic.size() + is_add.size(),
                Message::UnicastData(client_id, topic, data_packets) => {
                    client_id.size() + topic.size() + data_packets.size()
                }
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
        let initial = Message::AuthenticationRequest("basic".into(), "mary".into());

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).expect("should deserialize");
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_authentication_response() {
        let initial =
            Message::AuthenticationResponse("67e55044-10b1-426f-9247-bb680e5fe0c8".into());

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).expect("should deserialize");
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_multicast_data() {
        let initial = Message::ForwardedMulticastData(
            "host1".into(),
            "mary".into(),
            "VOD LSE".into(),
            vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        );

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_subscription_request() {
        let initial = Message::ForwardedSubscriptionRequest(
            "host1".into(),
            "mary".into(),
            "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            "VOD LSE".into(),
            true,
        );

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_unicast_data() {
        let initial = Message::ForwardedUnicastData(
            "host1".into(),
            "mary".into(),
            "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            "VOD LSE".into(),
            vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        );

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_multicast_data() {
        let initial = Message::MulticastData(
            "VOD LSE".into(),
            vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        );

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_notification_request() {
        let initial = Message::NotificationRequest(".* LSE".into(), true);

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_subscription_request() {
        let initial = Message::SubscriptionRequest("VOD LSE".into(), true);

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_unicast_data() {
        let initial = Message::UnicastData(
            "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            "VOD LSE".into(),
            vec![DataPacket {
                name: "greeting".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Hello, World!".into(),
            }],
        );

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }
}
