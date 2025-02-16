use std::io::Cursor;

use futures_util::{SinkExt, StreamExt};
use tokio::io::{self, AsyncRead, AsyncWrite};
use tokio_tungstenite::{tungstenite, WebSocketStream};

use crate::{message_stream::MessageStream, messages::Message, Serializable};

pub struct MessageWebSocket<T> {
    stream: WebSocketStream<T>,
}

impl<T> MessageWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: WebSocketStream<T>) -> MessageWebSocket<T> {
        MessageWebSocket { stream }
    }
}

impl<T> MessageStream for MessageWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    async fn read(&mut self) -> io::Result<Message> {
        let Some(result) = self.stream.next().await else {
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
        self.stream
            .send(tungstenite::Message::Binary(cursor.into_inner().into()))
            .await
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to send message: {}", e),
                )
            })
    }
}
