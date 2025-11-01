use std::{
    collections::{HashMap, HashSet},
    io::{self, Cursor, Read, Write},
};

pub trait Serializable: Sized + Send {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()>;
    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self>;
    fn size(&self) -> usize;
}

impl Serializable for u8 {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        let buf = [*self; 1];
        writer.write_all(&buf)?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let mut buf = [0_u8; 1];
        reader.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    fn size(&self) -> usize {
        1
    }
}

impl Serializable for bool {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        let value: u8 = if *self { 1 } else { 2 };
        value.serialize(writer)
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let value: u8 = u8::deserialize(reader)?;
        Ok(value == 1)
    }

    fn size(&self) -> usize {
        1
    }
}

impl Serializable for u32 {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        let buf = self.to_be_bytes();
        writer.write_all(&buf)
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let mut buf = [0_u8; 4];
        reader.read_exact(&mut buf)?;
        let i = Self::from_be_bytes(buf);
        Ok(i)
    }

    fn size(&self) -> usize {
        4
    }
}

impl Serializable for i32 {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        let buf = self.to_be_bytes();
        writer.write_all(&buf)
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let mut buf = [0_u8; 4];
        reader.read_exact(&mut buf)?;
        let i = Self::from_be_bytes(buf);
        Ok(i)
    }

    fn size(&self) -> usize {
        4
    }
}

impl Serializable for String {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        (self.len() as u32).serialize(writer)?;
        writer.write_all(self.as_bytes().into())?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let len = u32::deserialize(reader)?;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        match String::from_utf8(buf) {
            Ok(value) => Ok(value),
            Err(error) => Err(io::Error::new(io::ErrorKind::Other, error)),
        }
    }

    fn size(&self) -> usize {
        let len = self.len();
        (len as u32).size() + len
    }
}

impl Serializable for Vec<u8> {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        (self.len() as u32).serialize(writer)?;
        writer.write_all(self.as_slice())?;
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let len = u32::deserialize(reader)?;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn size(&self) -> usize {
        let len = self.len();
        (len as u32).size() + len
    }
}

impl Serializable for HashSet<i32> {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        (self.len() as u32).serialize(writer)?;
        for value in self {
            value.serialize(writer)?;
        }
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let len = u32::deserialize(reader)?;
        let capacity: usize = len
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut hash_set: HashSet<i32> = HashSet::with_capacity(capacity);
        for _ in 0..len {
            let value = i32::deserialize(reader)?;
            hash_set.insert(value);
        }
        Ok(hash_set)
    }

    fn size(&self) -> usize {
        let len = self.len();
        (len as u32).size() + size_of::<i32>() * len
    }
}

impl Serializable for HashMap<String, String> {
    fn serialize(&self, writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        (self.len() as u32).serialize(writer)?;
        for (key, value) in self {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }
        Ok(())
    }

    fn deserialize(reader: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        let len = u32::deserialize(reader)?;
        let capacity: usize = len
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut hash_map: HashMap<String, String> = HashMap::with_capacity(capacity);
        for _ in 0..len {
            let key = String::deserialize(reader)?;
            let value = String::deserialize(reader)?;
            hash_map.insert(key, value);
        }
        Ok(hash_map)
    }

    fn size(&self) -> usize {
        let mut len = self.len();
        for (key, value) in self {
            len += key.size();
            len += value.size();
        }
        len
    }
}

#[cfg(test)]
mod tests {
    use std::io::Seek;

    use super::*;

    #[test]
    fn should_roundtrip_u32() {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let actual: u32 = 12345678;
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match u32::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_pos_i32() {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let actual: i32 = 12345678;
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match i32::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_neg_i32() {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let actual: i32 = -12345678;
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match i32::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_string() {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let actual = String::from("Hello, World!");
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match String::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_i32_hash_set() {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let actual: HashSet<i32> =
            HashSet::from([-10000, -100, -10, -1, 0, 1, 10, 100, 1000, 10000]);
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match HashSet::<i32>::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_i32_hash_map() {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let actual: HashMap<String, String> = HashMap::from([
            ("a".to_string(), "one".to_string()),
            ("b".to_string(), "two".to_string()),
        ]);
        actual.serialize(&mut cursor).expect("should serialize");

        cursor.rewind().expect("should rewind");
        match HashMap::<String, String>::deserialize(&mut cursor) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }
}
