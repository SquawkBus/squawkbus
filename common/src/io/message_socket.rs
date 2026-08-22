use async_trait::async_trait;
use std::io::Cursor;

use tokio::io::{
    self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadHalf, WriteHalf,
};

use crate::{Serializable, message_stream::MessageStream, messages::Message};

pub struct MessageSocket<T> {
    reader: BufReader<ReadHalf<T>>,
    writer: WriteHalf<T>,
}

impl<T> MessageSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: T) -> MessageSocket<T> {
        let (read_half, writer) = tokio::io::split(stream);
        let reader = BufReader::new(read_half);
        MessageSocket { reader, writer }
    }
}

#[async_trait]
impl<T> MessageStream for MessageSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn read(&mut self) -> io::Result<Message> {
        // Read the frame length.
        let mut len_buf = [0_u8; 4];
        self.reader.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);

        log::trace!("Reading a frame of {len} bytes.");

        // Read the frame with the known length.
        let mut buf: Vec<u8> = vec![0; len as usize];
        self.reader.read_exact(&mut buf).await?;
        let mut cursor = Cursor::new(buf);

        // Parse the message.
        Message::deserialize(&mut cursor)
    }

    async fn write(&mut self, message: &Message) -> io::Result<()> {
        let len = message.size();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(4 + len));

        // Write the length of the frame into the cursor, then write the message.
        (len as u32).serialize(&mut cursor)?;
        message.serialize(&mut cursor)?;

        log::trace!("Writing a frame of {len} bytes.");

        self.writer.write_all(cursor.get_ref().as_slice()).await
    }
}
