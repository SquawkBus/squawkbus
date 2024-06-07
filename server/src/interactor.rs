use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};

use uuid::Uuid;

use common::messages::Message;

use crate::events::{ClientEvent, ServerEvent};

#[derive(Debug)]
pub struct Interactor {
    pub id: Uuid,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor { id: Uuid::new_v4() }
    }

    pub async fn run(
        &self,
        mut socket: TcpStream,
        addr: SocketAddr,
        hub: Sender<ClientEvent>,
    ) -> io::Result<()> {
        let (tx, mut rx) = mpsc::channel::<Arc<ServerEvent>>(32);

        let (read_half, mut write_half) = socket.split();

        let mut reader = BufReader::new(read_half);

        let (user, _password) = handshake(&mut reader).await.unwrap();

        let host = match addr {
            SocketAddr::V4(v4) => v4.ip().to_string(),
            SocketAddr::V6(v6) => v6.ip().to_string(),
        };

        // Inform the client
        hub.send(ClientEvent::OnConnect(self.id.clone(), host, user, tx))
            .await
            .unwrap();

        loop {
            tokio::select! {
                // forward client to hub
                result = Message::read(&mut reader) => {
                    self.forward_client_to_hub(result, &hub).await
                }
                // forward hub to client
                result = rx.recv() => {
                    forward_hub_to_client(result, &mut write_half).await
                }
            }?
        }
    }

    async fn forward_client_to_hub(
        &self,
        result: Result<Message, std::io::Error>,
        hub: &Sender<ClientEvent>,
    ) -> io::Result<()> {
        // let msg = result?;
        hub.send(ClientEvent::OnMessage(self.id, result?))
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }
}

async fn forward_hub_to_client<'a>(
    result: Option<Arc<ServerEvent>>,
    write_half: &mut WriteHalf<'a>,
) -> io::Result<()> {
    let event = result.ok_or(io::Error::new(io::ErrorKind::Other, "missing event"))?;
    match event.as_ref() {
        ServerEvent::OnMessage(message) => {
            message.write(write_half).await?;
        }
    }

    Ok(())
}

async fn handshake<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> Result<(String, String), tokio::io::Error> {
    let mut user = String::new();
    reader.read_line(&mut user).await?;
    user.truncate(user.len() - 1); // Must have at least a single '\n';
    let mut password = String::new();
    reader.read_line(&mut password).await?;
    password.truncate(password.len() - 1); // Must have at least a single '\n';
    Ok((user, password))
}
