use std::io::Cursor;

use common::messages::Message;
use common::Serializable;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{self, AsyncRead, AsyncWrite};
use tokio_tungstenite::{tungstenite, WebSocketStream};

use crate::message_stream::MessageStream;

impl<T> MessageStream for WebSocketStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    async fn read(&mut self) -> io::Result<Message> {
        let Some(result) = self.next().await else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to receive ws message",
            ));
        };
        let message = result.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to receive ws message: {}", e),
            )
        })?;

        match message {
            tungstenite::Message::Binary(buf) => {
                let mut cursor: Cursor<Vec<u8>> = Cursor::new(buf.into());
                Message::deserialize(&mut cursor)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to receive message",
            )),
        }
    }

    async fn write(&mut self, message: &Message) -> io::Result<()> {
        let bytes_to_write = message.size();
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(bytes_to_write));
        message.serialize(&mut cursor)?;
        self.send(tungstenite::Message::Binary(cursor.into_inner().into()))
            .await
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to send message: {}", e),
                )
            })
    }
}
