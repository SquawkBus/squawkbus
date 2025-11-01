use std::io::{self, Cursor};

use crate::io::Serializable;

use super::message_type::MessageType;

use super::DataPacket;

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    AuthenticationRequest {
        method: String,
        credentials: Vec<u8>,
    },
    AuthenticationResponse {
        client_id: String,
    },
    ForwardedMulticastData {
        host: String,
        user: String,
        topic: String,
        data_packets: Vec<DataPacket>,
    },
    ForwardedSubscriptionRequest {
        host: String,
        user: String,
        client_id: String,
        topic: String,
        count: u32,
    },
    ForwardedUnicastData {
        host: String,
        user: String,
        client_id: String,
        topic: String,
        data_packets: Vec<DataPacket>,
    },
    MulticastData {
        topic: String,
        data_packets: Vec<DataPacket>,
    },
    SubscriptionRequest {
        topic: String,
        is_add: bool,
    },
    UnicastData {
        client_id: String,
        topic: String,
        data_packets: Vec<DataPacket>,
    },
}

impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::AuthenticationRequest { .. } => MessageType::AuthenticationRequest,
            Message::AuthenticationResponse { .. } => MessageType::AuthenticationResponse,
            Message::ForwardedMulticastData { .. } => MessageType::ForwardedMulticastData,
            Message::ForwardedSubscriptionRequest { .. } => {
                MessageType::ForwardedSubscriptionRequest
            }
            Message::ForwardedUnicastData { .. } => MessageType::ForwardedUnicastData,
            Message::MulticastData { .. } => MessageType::MulticastData,
            Message::SubscriptionRequest { .. } => MessageType::SubscriptionRequest,
            Message::UnicastData { .. } => MessageType::UnicastData,
        }
    }
}

impl Serializable for Message {
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Message> {
        match MessageType::deserialize(reader) {
            Ok(MessageType::AuthenticationRequest) => {
                let method = String::deserialize(reader)?;
                let credentials = Vec::deserialize(reader)?;
                Ok(Message::AuthenticationRequest {
                    method,
                    credentials,
                })
            }
            Ok(MessageType::AuthenticationResponse) => {
                let client_id = String::deserialize(reader)?;
                Ok(Message::AuthenticationResponse { client_id })
            }
            Ok(MessageType::ForwardedMulticastData) => {
                let host = String::deserialize(reader)?;
                let user = String::deserialize(reader)?;
                let topic = String::deserialize(reader)?;
                let data_packets = Vec::<DataPacket>::deserialize(reader)?;
                Ok(Message::ForwardedMulticastData {
                    host,
                    user,
                    topic,
                    data_packets,
                })
            }
            Ok(MessageType::ForwardedSubscriptionRequest) => {
                let host = String::deserialize(reader)?;
                let user = String::deserialize(reader)?;
                let client_id = String::deserialize(reader)?;
                let topic = String::deserialize(reader)?;
                let count = u32::deserialize(reader)?;
                Ok(Message::ForwardedSubscriptionRequest {
                    host,
                    user,
                    client_id,
                    topic,
                    count,
                })
            }
            Ok(MessageType::ForwardedUnicastData) => {
                let host = String::deserialize(reader)?;
                let user = String::deserialize(reader)?;
                let client_id = String::deserialize(reader)?;
                let topic = String::deserialize(reader)?;
                let data_packets = Vec::<DataPacket>::deserialize(reader)?;
                Ok(Message::ForwardedUnicastData {
                    host,
                    user,
                    client_id,
                    topic,
                    data_packets,
                })
            }
            Ok(MessageType::MulticastData) => {
                let topic = String::deserialize(reader)?;
                let data_packets = Vec::<DataPacket>::deserialize(reader)?;
                Ok(Message::MulticastData {
                    topic,
                    data_packets,
                })
            }
            Ok(MessageType::SubscriptionRequest) => {
                let topic = String::deserialize(reader)?;
                let is_add = bool::deserialize(reader)?;
                Ok(Message::SubscriptionRequest { topic, is_add })
            }
            Ok(MessageType::UnicastData) => {
                let client_id = String::deserialize(reader)?;
                let topic = String::deserialize(reader)?;
                let data_packets = Vec::<DataPacket>::deserialize(reader)?;
                Ok(Message::UnicastData {
                    client_id,
                    topic,
                    data_packets,
                })
            }
            Err(error) => Err(error),
        }
    }

    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.message_type().serialize(writer)?;
        match self {
            Message::AuthenticationRequest {
                method,
                credentials,
            } => {
                method.serialize(writer)?;
                credentials.serialize(writer)?;
                Ok(())
            }
            Message::AuthenticationResponse { client_id } => {
                client_id.serialize(writer)?;
                Ok(())
            }
            Message::ForwardedMulticastData {
                host,
                user,
                topic,
                data_packets,
            } => {
                host.serialize(writer)?;
                user.serialize(writer)?;
                topic.serialize(writer)?;
                data_packets.serialize(writer)?;
                Ok(())
            }
            Message::ForwardedSubscriptionRequest {
                host,
                user,
                client_id,
                topic,
                count,
            } => {
                host.serialize(writer)?;
                user.serialize(writer)?;
                client_id.serialize(writer)?;
                topic.serialize(writer)?;
                count.serialize(writer)?;
                Ok(())
            }
            Message::ForwardedUnicastData {
                host,
                user,
                client_id,
                topic,
                data_packets,
            } => {
                host.serialize(writer)?;
                user.serialize(writer)?;
                client_id.serialize(writer)?;
                topic.serialize(writer)?;
                data_packets.serialize(writer)?;
                Ok(())
            }
            Message::MulticastData {
                topic,
                data_packets,
            } => {
                topic.serialize(writer)?;
                data_packets.serialize(writer)?;
                Ok(())
            }
            Message::SubscriptionRequest { topic, is_add } => {
                topic.serialize(writer)?;
                is_add.serialize(writer)?;
                Ok(())
            }
            Message::UnicastData {
                client_id,
                topic,
                data_packets,
            } => {
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
                Message::AuthenticationRequest {
                    method,
                    credentials,
                } => method.size() + credentials.size(),
                Message::AuthenticationResponse { client_id } => client_id.size(),
                Message::ForwardedMulticastData {
                    host,
                    user,
                    topic,
                    data_packets,
                } => host.size() + user.size() + topic.size() + data_packets.size(),
                Message::ForwardedSubscriptionRequest {
                    host,
                    user,
                    client_id,
                    topic,
                    count,
                } => host.size() + user.size() + client_id.size() + topic.size() + count.size(),
                Message::ForwardedUnicastData {
                    host,
                    user,
                    client_id,
                    topic,
                    data_packets,
                } => {
                    host.size()
                        + user.size()
                        + client_id.size()
                        + topic.size()
                        + data_packets.size()
                }
                Message::MulticastData {
                    topic,
                    data_packets,
                } => topic.size() + data_packets.size(),
                Message::SubscriptionRequest { topic, is_add } => topic.size() + is_add.size(),
                Message::UnicastData {
                    client_id,
                    topic,
                    data_packets,
                } => client_id.size() + topic.size() + data_packets.size(),
            }
    }
}

#[cfg(test)]
mod test_message {
    use super::super::data_packet::DataPacket;
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::io::Seek;

    #[test]
    fn should_round_trip_authentication_request() {
        let initial = Message::AuthenticationRequest {
            method: "basic".into(),
            credentials: "mary".into(),
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).expect("should deserialize");
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_authentication_response() {
        let initial = Message::AuthenticationResponse {
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).expect("should deserialize");
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_multicast_data() {
        let initial = Message::ForwardedMulticastData {
            host: "host1".into(),
            user: "mary".into(),
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
                entitlements: HashSet::from([1]),
                data: "Hello, World!".into(),
            }],
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_subscription_request() {
        let initial = Message::ForwardedSubscriptionRequest {
            host: "host1".into(),
            user: "mary".into(),
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            topic: "VOD LSE".into(),
            count: 1,
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_forwarded_unicast_data() {
        let initial = Message::ForwardedUnicastData {
            host: "host1".into(),
            user: "mary".into(),
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
                entitlements: HashSet::from([1]),
                data: "Hello, World!".into(),
            }],
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_multicast_data() {
        let initial = Message::MulticastData {
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
                entitlements: HashSet::from([1]),
                data: "Hello, World!".into(),
            }],
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_subscription_request() {
        let initial = Message::SubscriptionRequest {
            topic: "VOD LSE".into(),
            is_add: true,
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }

    #[test]
    fn should_roundtrip_unicast_data() {
        let initial = Message::UnicastData {
            client_id: "67e55044-10b1-426f-9247-bb680e5fe0c8".into(),
            topic: "VOD LSE".into(),
            data_packets: vec![DataPacket {
                headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
                entitlements: HashSet::from([1]),
                data: "Hello, World!".into(),
            }],
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        initial.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        let round_trip = Message::deserialize(&mut cursor).unwrap();
        assert_eq!(initial, round_trip);
    }
}
