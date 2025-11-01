use std::collections::{HashMap, HashSet};
use std::io::{self, Cursor};

use crate::io::Serializable;

#[derive(Debug, PartialEq, Clone)]
pub struct DataPacket {
    pub entitlements: HashSet<i32>,
    pub headers: HashMap<String, String>,
    pub data: Vec<u8>,
}

impl DataPacket {
    pub fn new(
        entitlements: HashSet<i32>,
        headers: HashMap<String, String>,
        data: Vec<u8>,
    ) -> DataPacket {
        DataPacket {
            entitlements,
            headers,
            data,
        }
    }

    pub fn is_authorized(&self, all_entitlements: &HashSet<i32>) -> bool {
        all_entitlements.is_superset(&self.entitlements)
    }
}

impl Serializable for DataPacket {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        self.entitlements.serialize(writer)?;
        self.headers.serialize(writer)?;
        self.data.serialize(writer)?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<DataPacket> {
        let entitlements = HashSet::<i32>::deserialize(reader)?;
        let headers = HashMap::<String, String>::deserialize(reader)?;
        let data = Vec::<u8>::deserialize(reader)?;
        Ok(DataPacket::new(entitlements, headers, data))
    }

    fn size(&self) -> usize {
        self.entitlements.size() + self.headers.size() + self.data.size()
    }
}

impl Serializable for Vec<DataPacket> {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        (self.len() as u32).serialize(writer)?;
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
        let mut len = (self.len() as u32).size();
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
            entitlements: HashSet::from([1]),
            headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
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
                entitlements: HashSet::from([1]),
                headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
                data: "Data 1".into(),
            },
            DataPacket {
                entitlements: HashSet::from([2]),
                headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
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

    #[test]
    fn check_is_authorized() {
        // Same single entitlement.
        let data_packet = DataPacket {
            entitlements: HashSet::from([1]),
            headers: HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]),
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
