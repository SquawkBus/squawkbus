use std::io;

use super::frame::{FrameReader, FrameWriter};

pub trait Serializable: Sized + Send {
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()>;
    fn deserialize(reader: &mut FrameReader) -> io::Result<Self>;
}

impl Serializable for u8 {
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        writer.push(vec![*self])?;

        Ok(())
    }

    fn deserialize(reader: &mut FrameReader) -> io::Result<Self> {
        let mut buf = [0_u8; 1];
        reader.take(&mut buf)?;

        Ok(buf[0])
    }
}

impl Serializable for bool {
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        if *self {
            writer.push(vec![1])?;
        } else {
            writer.push(vec![0])?;
        }

        Ok(())
    }

    fn deserialize(reader: &mut FrameReader) -> io::Result<Self> {
        let mut buf = [0_u8; 1];
        reader.take(&mut buf)?;

        Ok(buf[0] == 1)
    }
}

impl Serializable for u32 {
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        let buf = self.to_be_bytes();
        writer.push(buf.into())?;
        Ok(())
    }

    fn deserialize(reader: &mut FrameReader) -> io::Result<Self> {
        let mut buf = [0_u8; 4];
        reader.take(&mut buf)?;
        let i = Self::from_be_bytes(buf);
        Ok(i)
    }
}

impl Serializable for i32 {
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        let buf = self.to_be_bytes();
        writer.push(buf.into())?;
        Ok(())
    }

    fn deserialize(reader: &mut FrameReader) -> io::Result<Self> {
        let mut buf = [0_u8; 4];
        reader.take(&mut buf)?;
        let i = Self::from_be_bytes(buf);
        Ok(i)
    }
}

impl Serializable for String {
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        let len_buf = (self.len() as u32).to_be_bytes();
        writer.push(len_buf.into())?;
        writer.push(self.as_bytes().into())?;
        Ok(())
    }

    fn deserialize(reader: &mut FrameReader) -> io::Result<Self> {
        let len = u32::deserialize(reader)?;
        let mut buf = vec![0u8; len as usize];
        reader.take(&mut buf)?;
        match String::from_utf8(buf) {
            Ok(value) => Ok(value),
            Err(error) => Err(io::Error::new(io::ErrorKind::Other, error)),
        }
    }
}

impl<T> Serializable for Vec<T>
where
    T: Serializable,
{
    fn serialize(&self, writer: &mut FrameWriter) -> io::Result<()> {
        let len_buf = (self.len() as u32).to_be_bytes();
        writer.push(len_buf.into())?;
        for value in self {
            value.serialize(writer)?;
        }
        Ok(())
    }

    fn deserialize(reader: &mut FrameReader) -> io::Result<Self> {
        let mut len = u32::deserialize(reader)?;
        let mut buf = Vec::with_capacity(len as usize);
        while len > 0 {
            let value = T::deserialize(reader)?;
            buf.push(value);
            len = len - 1;
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_roundtrip_u32() {
        let mut writer = FrameWriter::new();

        let actual: u32 = 12345678;
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match u32::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_pos_i32() {
        let mut writer = FrameWriter::new();

        let actual: i32 = 12345678;
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match i32::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_neg_i32() {
        let mut writer = FrameWriter::new();

        let actual: i32 = -12345678;
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match i32::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_string() {
        let mut writer = FrameWriter::new();

        let actual = String::from("Hello, World!");
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match String::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_u32_vec() {
        let mut writer = FrameWriter::new();

        let actual: Vec<u32> = vec![1, 10, 100, 1000, 10000];
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match Vec::<u32>::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_i32_vec() {
        let mut writer = FrameWriter::new();

        let actual: Vec<i32> = vec![-10000, -100, -10, -1, 0, 1, 10, 100, 1000, 10000];
        actual.serialize(&mut writer).expect("should serialize");

        let mut reader = FrameReader::from(&writer);
        match Vec::<i32>::deserialize(&mut reader) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }
}
