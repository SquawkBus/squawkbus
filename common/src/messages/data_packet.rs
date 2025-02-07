use std::collections::HashSet;
use std::io::{self, Cursor};

use crate::io::Serializable;

#[derive(Debug, PartialEq, Clone)]
pub struct DataPacket {
    pub name: String,
    pub entitlement: i32,
    pub content_type: String,
    pub data: Vec<u8>,
}

impl DataPacket {
    pub fn new(name: String, entitlement: i32, content_type: String, data: Vec<u8>) -> DataPacket {
        DataPacket {
            name,
            entitlement,
            content_type,
            data,
        }
    }

    pub fn is_authorized(&self, all_entitlements: &HashSet<i32>) -> bool {
        all_entitlements.contains(&self.entitlement)
    }
}

impl Serializable for DataPacket {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.name.serialize(writer)?;
        self.entitlement.serialize(writer)?;
        self.content_type.serialize(writer)?;
        self.data.serialize(writer)?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<DataPacket> {
        let name = String::deserialize(reader)?;
        let entitlement = i32::deserialize(reader)?;
        let content_type = String::deserialize(reader)?;
        let data = Vec::<u8>::deserialize(reader)?;
        Ok(DataPacket::new(name, entitlement, content_type, data))
    }

    fn size(&self) -> usize {
        self.name.size() + self.entitlement.size() + self.content_type.size() + self.data.size()
    }
}

impl Serializable for Vec<DataPacket> {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        (self.len() as u32).serialize(writer);
        for value in self {
            value.serialize(writer)?;
        }
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let mut len = u32::deserialize(reader)?;
        let mut buf = Vec::with_capacity(len as usize);
        while len > 0 {
            let value = DataPacket::deserialize(reader)?;
            buf.push(value);
            len = len - 1;
        }
        Ok(buf)
    }

    fn size(&self) -> usize {
        let mut len = self.len();
        for value in self {
            len = len + value.size()
        }
        len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Seek;

    #[test]
    fn should_roundtrip_datapacket() {
        let actual = DataPacket {
            name: "image".into(),
            entitlement: 1,
            content_type: "text/plain".into(),
            data: "Hello, World!".into(),
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match DataPacket::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_vec_datapacket() {
        let actual = vec![
            DataPacket {
                name: "Level 1".into(),
                entitlement: 1,
                content_type: "text/plain".into(),
                data: "Data 1".into(),
            },
            DataPacket {
                name: "Level 2".into(),
                entitlement: 2,
                content_type: "text/plain".into(),
                data: "Data 2".into(),
            },
        ];

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match Vec::<DataPacket>::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn check_is_authorized() {
        // Same single entitlement.
        let data_packet = DataPacket {
            name: "Level 1".into(),
            entitlement: 1,
            content_type: "text/plain".into(),
            data: "Data 1".into(),
        };

        let has_entitlement = HashSet::from([1i32]);

        assert!(data_packet.is_authorized(&has_entitlement));

        let wrong_entitlements = HashSet::from([2i32]);

        assert!(!data_packet.is_authorized(&wrong_entitlements));

        let empty_entitlements = HashSet::new();

        assert!(!data_packet.is_authorized(&empty_entitlements));
    }
}
