use std::collections::HashSet;
use std::hash::Hash;

use tokio::io::{self,AsyncReadExt,AsyncWriteExt};

use uuid::Uuid;

#[allow(async_fn_in_trait)]
//#[trait_variant::make(HttpService: Send)]
pub trait Serializable: Sized+Send {
    async fn write<W: AsyncWriteExt+Unpin>(&self, writer: &mut W) -> io::Result<()>;
    async fn read<R: AsyncReadExt+Unpin>(reader: &mut R) -> io::Result<Self>;
}

impl Serializable for u8 {
    async fn write<W: AsyncWriteExt+Unpin>(&self, writer: &mut W) -> io::Result<()> {
        let buf = [*self];
        writer.write_all(&buf).await
    }

    async fn read<R: AsyncReadExt+Unpin>(reader: &mut R) -> io::Result<u8> {
        let mut buf: [u8; 1] = [0];
        reader.read_exact(&mut buf).await?;
        Ok(buf[0])
    }
}

impl Serializable for bool {
    async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        if *self {
            1u8.write(&mut writer).await?;
        } else {
            1u8.write(&mut writer).await?;
        }
    
        Ok(())            
    }

    async fn read<R: AsyncReadExt+Unpin>(mut reader: &mut R) -> io::Result<bool> {
        match u8::read(&mut reader).await {
            Ok(value) => if value == 1 { Ok(true) } else { Ok(false) },
            Err(error) => Err(error)
        }
    }

}

impl Serializable for u32 {
    async fn write<W: AsyncWriteExt+Unpin>(&self, writer: &mut W) -> io::Result<()> {
        let buf: [u8; 4] = [
            *self as u8,
            (*self >> 8) as u8,
            (*self >> 16) as u8,
            (*self >> 24) as u8
        ];
        writer.write(&buf).await?;
    
        Ok(())            
    }

    async fn read<R: AsyncReadExt+Unpin>(reader: &mut R) -> io::Result<u32> {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        reader.read_exact(&mut buf).await?;
        let value: u32 = buf[0] as u32 |
            (buf[1] as u32) << 8 |
            (buf[2] as u32) << 16 |
            (buf[3] as u32) << 24;
        Ok(value)
    }
}

impl Serializable for i32 {
    async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        (*self as u32).write(&mut writer).await
    }

    async fn read<R: AsyncReadExt+Unpin>(mut reader: &mut R) -> io::Result<i32> {
        match u32::read(&mut reader).await {
            Ok(num) => Ok(num as i32),
            Err(error) => Err(error)
        }
    }
}

impl Serializable for String {
    async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        (self.len() as u32).write(&mut writer).await?;
        writer.write(self.as_bytes()).await?;
        Ok(())
    }

    async fn read<R: AsyncReadExt+Unpin>(mut reader: &mut R) -> io::Result<String> {
        let len = u32::read(&mut reader).await?;
        let mut buf = vec![0u8; len as usize];
        reader.read(&mut buf).await?;
        match String::from_utf8(buf) {
            Ok(value) => Ok(value),
            Err(error) => Err(io::Error::new(io::ErrorKind::Other, error)),
        }
    }
}

impl<T> Serializable for Vec<T> where T: Serializable {
    async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        (self.len() as u32).write(&mut writer).await?;
        for value in self {
            value.write(&mut writer).await?;
        }
        Ok(())
    }

    async fn read<R: AsyncReadExt+Unpin>(mut reader: &mut R) -> io::Result<Self> {
        let mut len = u32::read(&mut reader).await?;
        let mut values: Self = Vec::new();
        values.reserve_exact(len as usize);
        while len > 0 {
            let value = T::read(&mut reader).await?;
            values.push(value);
            len = len - 1;
        }
        Ok(values)
    }
}

impl<T> Serializable for HashSet<T> where T: Serializable + Eq + Hash {
    async fn write<W: AsyncWriteExt+Unpin>(&self, mut writer: &mut W) -> io::Result<()> {
        (self.len() as u32).write(&mut writer).await?;
        for value in self {
            value.write(&mut writer).await?;
        }
        Ok(())
    }

    async fn read<R: AsyncReadExt+Unpin>(mut reader: &mut R) -> io::Result<Self> {
        let mut len = u32::read(&mut reader).await?;
        let mut values: Self = HashSet::new();
        while len > 0 {
            let value = T::read(&mut reader).await?;
            values.insert(value);
            len = len - 1;
        }
        Ok(values)
    }
}

impl Serializable for Uuid {
    async fn read<R: AsyncReadExt+Unpin>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 16];
        reader.read_exact(&mut buf).await?;
        let value = Uuid::from_bytes(buf);
        Ok(value)
    }

    async fn write<W: AsyncWriteExt+Unpin>(&self, writer: &mut W) -> io::Result<()> {
        writer.write(self.as_bytes()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek};

    #[tokio::test]
    async fn should_roundtrip_u32() {
        let mut buf = Cursor::new(Vec::new());

        let actual: u32 = 12345678;
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match u32::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_pos_i32() {
        let mut buf = Cursor::new(Vec::new());

        let actual: i32 = 12345678;
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match i32::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_neg_i32() {
        let mut buf = Cursor::new(Vec::new());

        let actual: i32 = -12345678;
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match i32::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_string() {
        let mut buf = Cursor::new(Vec::new());

        let actual = String::from("Hello, World!");
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match String::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_u32_vec() {
        let mut buf = Cursor::new(Vec::new());

        let actual: Vec<u32> = vec![1, 10, 100, 1000, 10000];
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match Vec::<u32>::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_i32_vec() {
        let mut buf = Cursor::new(Vec::new());

        let actual: Vec<i32> = vec![-10000, -100, -10, -1, 0, 1, 10, 100, 1000, 10000];
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match Vec::<i32>::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_i32_hashset() {
        let mut buf = Cursor::new(Vec::new());

        let mut actual: HashSet<i32> = HashSet::new();
        actual.insert(1);
        actual.insert(3);
        actual.insert(5);
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match HashSet::<i32>::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[tokio::test]
    async fn should_roundtrip_uuid() {
        let mut buf = Cursor::new(Vec::new());

        let actual = Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").expect("Should parse");
        actual.write(&mut buf).await.expect("should serialize");
        buf.seek(io::SeekFrom::Start(0)).unwrap();
        match Uuid::read(&mut buf).await {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }
}
