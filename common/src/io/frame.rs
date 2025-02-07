use std::io::{Cursor, IoSlice};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

pub struct FrameWriter {
    buffers: Vec<Vec<u8>>,
    length: u32,
}

impl FrameWriter {
    pub fn new() -> FrameWriter {
        FrameWriter {
            buffers: Vec::new(),
            length: 0,
        }
    }

    pub fn from(buf: Vec<u8>) -> io::Result<FrameWriter> {
        let mut frame = FrameWriter::new();
        frame.push(buf)?;
        Ok(frame)
    }

    fn check_requested_size(&self, requested_bytes: usize) -> io::Result<()> {
        if requested_bytes > (u32::MAX - self.length) as usize {
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "exceeds maximum size",
            ));
        }
        Ok(())
    }

    pub fn push(&mut self, buf: Vec<u8>) -> io::Result<&FrameWriter> {
        self.check_requested_size(buf.len())?;
        self.length += buf.len() as u32;
        self.buffers.push(buf);
        Ok(self)
    }

    pub fn pack(&self) -> Vec<u8> {
        let len = self.length;
        let mut buffer: Vec<u8> = Vec::with_capacity(4 + self.length as usize);
        buffer.extend_from_slice(&len.to_be_bytes());
        for buf in &self.buffers {
            buffer.extend_from_slice(buf);
        }
        buffer
    }

    pub async fn write<W: AsyncWriteExt + Unpin>(&self, writer: &mut W) -> io::Result<()> {
        // We will need one more buffer to hold the length.
        let mut io_slices: Vec<IoSlice> = Vec::with_capacity(1 + self.buffers.len());

        let bytes_to_write = self.length as usize;
        let len_buf = (bytes_to_write as u32).to_be_bytes();
        io_slices.push(IoSlice::new(&len_buf));

        for buffer in &self.buffers {
            let buf: IoSlice = IoSlice::new(&buffer);
            if buf.len() > 0 {
                io_slices.push(buf);
            }
        }

        let mut total_bytes_written = 0;
        while total_bytes_written < bytes_to_write {
            let bytes_written = writer.write_vectored(&io_slices).await?;
            IoSlice::advance_slices(&mut io_slices.as_mut_slice(), bytes_written);
            total_bytes_written += bytes_written;
        }

        writer.flush().await?;

        Ok(())
    }
}

pub struct FrameReader {
    cursor: Cursor<Vec<u8>>,
}

impl FrameReader {
    pub fn from(frame_writer: &FrameWriter) -> Self {
        let buf = frame_writer.pack();
        let cursor = Cursor::new(buf);
        FrameReader { cursor }
    }
    pub async fn read<R: AsyncReadExt + Unpin>(reader: &mut R) -> io::Result<FrameReader> {
        let mut len_buf = [0_u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);
        let mut buf: Vec<u8> = vec![0; len as usize];
        reader.read_exact(&mut buf).await?;
        let frame = FrameReader {
            cursor: Cursor::new(buf),
        };
        Ok(frame)
    }

    pub fn take(&mut self, buf: &mut [u8]) -> io::Result<()> {
        std::io::Read::read_exact(&mut self.cursor, buf)
    }
}
