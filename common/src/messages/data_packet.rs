use std::collections::HashSet;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::io::Serializable;

#[derive(Debug, PartialEq, Clone)]
pub struct DataPacket {
    pub entitlements: HashSet<i32>,
    pub data: Vec<u8>,
}

impl DataPacket {
    pub fn new(entitlements: HashSet<i32>, data: Vec<u8>) -> DataPacket {
        DataPacket { entitlements, data }
    }

    pub fn is_authorized(&self, all_entitlements: &HashSet<i32>) -> bool {
        all_entitlements.is_superset(&self.entitlements)
    }
}

impl Serializable for DataPacket {
    async fn write<W: AsyncWriteExt + Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        self.entitlements.write(&mut writer).await?;
        self.data.write(&mut writer).await?;
        Ok(())
    }

    async fn read<R: AsyncReadExt + Unpin>(mut reader: &mut R) -> io::Result<DataPacket> {
        let entitlements = HashSet::<i32>::read(&mut reader).await?;
        let data = Vec::<u8>::read(&mut reader).await?;
        let data_packet = DataPacket::new(entitlements, data);
        Ok(data_packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek};

    #[tokio::test]
    async fn should_roundtrip_datapacket() {
        let mut buf = Cursor::new(Vec::new());

        let actual = DataPacket {
            entitlements: HashSet::from([-5i32, 1, 17]),
            data: vec![1u8, 2, 3, 4],
        };

        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match DataPacket::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_vec_datapacket() {
        let mut buf = Cursor::new(Vec::new());

        let actual = vec![
            DataPacket {
                entitlements: HashSet::from([-5i32, 1, 17]),
                data: vec![1u8, 2, 3, 4],
            },
            DataPacket {
                entitlements: HashSet::from([12i32, 1, -22]),
                data: vec![100u8, 5, 32, 91],
            },
        ];

        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match Vec::<DataPacket>::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn check_is_authorized() {
        // Same single entitlement.
        let data_packet = DataPacket {
            entitlements: HashSet::from([1i32]),
            data: vec![1u8, 2, 3, 4],
        };

        let same_single_entitlement = HashSet::from([1i32]);

        assert!(data_packet.is_authorized(&same_single_entitlement));

        // Superset of entitlements
        let data_packet = DataPacket {
            entitlements: HashSet::from([1i32]),
            data: vec![1u8, 2, 3, 4],
        };

        let superset_entitlements = HashSet::from([1i32, 2, 3]);

        assert!(data_packet.is_authorized(&superset_entitlements));

        // Single wrong entitlement
        let data_packet = DataPacket {
            entitlements: HashSet::from([1i32]),
            data: vec![1u8, 2, 3, 4],
        };

        let wrong_entitlements = HashSet::from([2i32]);

        assert!(!data_packet.is_authorized(&wrong_entitlements));

        // Empty entitlement vs some entitlements
        let data_packet = DataPacket {
            entitlements: HashSet::new(),
            data: vec![1u8, 2, 3, 4],
        };

        let wrong_entitlements = HashSet::from([1i32, 2, 3]);

        assert!(data_packet.is_authorized(&wrong_entitlements));

        // Empty entitlement vs empty entitlements
        let data_packet = DataPacket {
            entitlements: HashSet::new(),
            data: vec![1u8, 2, 3, 4],
        };

        let wrong_entitlements = HashSet::new();

        assert!(data_packet.is_authorized(&wrong_entitlements));
    }
}
