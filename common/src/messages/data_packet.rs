use std::{collections::HashSet, io};

use crate::{
    frame::{FrameReader, FrameWriter},
    io::Serializable,
};

#[derive(Debug, PartialEq, Clone)]
pub struct DataPacket {
    pub name: String,
    pub content_type: String,
    pub entitlement: i32,
    pub data: Vec<u8>,
}

impl DataPacket {
    pub fn new(name: String, content_type: String, entitlement: i32, data: Vec<u8>) -> DataPacket {
        DataPacket {
            name,
            content_type,
            entitlement,
            data,
        }
    }

    pub fn is_authorized(&self, all_entitlements: &HashSet<i32>) -> bool {
        all_entitlements.contains(&self.entitlement)
    }
}

impl Serializable for DataPacket {
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        self.name.serialize(writer)?;
        self.content_type.serialize(writer)?;
        self.entitlement.serialize(writer)?;
        self.data.serialize(writer)?;
        Ok(())
    }

    fn deserialize(reader: &mut FrameReader) -> io::Result<DataPacket> {
        let name = String::deserialize(reader)?;
        let content_type = String::deserialize(reader)?;
        let entitlement = i32::deserialize(reader)?;
        let data = Vec::<u8>::deserialize(reader)?;
        let data_packet = DataPacket::new(name, content_type, entitlement, data);
        Ok(data_packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_roundtrip_datapacket() {
        let actual = DataPacket {
            name: "image".into(),
            content_type: "text/plain".into(),
            entitlement: 1,
            data: "Hello, World!".into(),
        };

        let mut writer = FrameWriter::new();
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match DataPacket::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_vec_datapacket() {
        let actual = vec![
            DataPacket {
                name: "Level 1".into(),
                content_type: "text/plain".into(),
                entitlement: 1,
                data: "Data 1".into(),
            },
            DataPacket {
                name: "Level 2".into(),
                content_type: "text/plain".into(),
                entitlement: 2,
                data: "Data 2".into(),
            },
        ];

        let mut writer = FrameWriter::new();
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match Vec::<DataPacket>::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn check_is_authorized() {
        // Same single entitlement.
        let data_packet = DataPacket {
            name: "Level 1".into(),
            content_type: "text/plain".into(),
            entitlement: 1,
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
