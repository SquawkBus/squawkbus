use std::collections::HashSet;
use std::io::prelude::*;
use std::io;

use crate::io::Serializable;

#[derive(PartialEq, Debug)]
pub struct DataPacket {
    pub entitlements: HashSet<i32>,
    pub data: Vec<u8>
}

impl DataPacket {
    pub fn new(entitlements: HashSet<i32>, data: Vec<u8>) -> DataPacket {
        DataPacket {
            entitlements,
            data
        }
    }

    pub fn is_authorized(&self, all_entitlements: &HashSet<i32>) -> bool {
        all_entitlements.is_superset(&self.entitlements)
    }
}

impl Serializable for DataPacket {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        self.entitlements.write(&mut writer)?;
        self.data.write(&mut writer)?;
        Ok(())
    }    

    fn read<R: Read>(mut reader: R) -> io::Result<DataPacket> {
        let entitlements = HashSet::<i32>::read(&mut reader)?;
        let data = Vec::<u8>::read(&mut reader)?;
        let data_packet = DataPacket::new(entitlements, data);
        Ok(data_packet)
    }
}

// impl Writeable for Vec<DataPacket> {
//     fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
//         (self.len() as u32).write(&mut writer)?;
//         for value in self {
//             value.write(&mut writer)?;
//         }
//         Ok(())
//     }
// }


// impl Readable for Vec<DataPacket> {

//     fn read<R: Read>(mut reader: R) -> io::Result<Self> {
//         let mut len = u32::read(&mut reader)?;
//         let mut values: Self = Vec::new();
//         values.reserve_exact(len as usize);
//         while len > 0 {
//             let value = DataPacket::read(&mut reader)?;
//             values.push(value);
//             len = len - 1;
//         }
//         Ok(values)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek};

    #[test]
    fn should_roundtrip_datapacket() {
        let mut buf = Cursor::new(Vec::new());

        let actual = DataPacket {
            entitlements: HashSet::from([-5i32, 1, 17]),
            data: vec![1u8, 2, 3, 4]
        };

        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match DataPacket::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_vec_datapacket() {
        let mut buf = Cursor::new(Vec::new());

        let actual = vec![
            DataPacket {
                entitlements: HashSet::from([-5i32, 1, 17]),
                data: vec![1u8, 2, 3, 4]
            },
            DataPacket {
                entitlements: HashSet::from([12i32, 1, -22]),
                data: vec![100u8, 5, 32, 91]
            },
        ];

        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match Vec::<DataPacket>::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }
}