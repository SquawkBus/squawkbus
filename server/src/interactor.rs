use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite, BufReader, WriteHalf};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::RwLock;

use uuid::Uuid;

use common::messages::Message;

use crate::authentication::AuthenticationManager;
use crate::events::{ClientEvent, ServerEvent};

#[derive(Debug)]
pub struct Interactor {
    pub id: Uuid,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor { id: Uuid::new_v4() }
    }

    pub async fn run<'a, T>(
        &self,
        socket: T,
        addr: SocketAddr,
        hub: Sender<ClientEvent>,
        authentication_manager: Arc<RwLock<AuthenticationManager>>,
    ) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite,
    {
        let (tx, mut rx) = mpsc::channel::<ServerEvent>(32);

        let (read_half, mut write_half) = tokio::io::split(socket);

        let mut reader = BufReader::new(read_half);

        let user = authentication_manager
            .read()
            .await
            .authenticate(&mut reader)
            .await?;

        let host = match addr {
            SocketAddr::V4(v4) => v4.ip().to_string(),
            SocketAddr::V6(v6) => v6.ip().to_string(),
        };

        // Inform the client
        hub.send(ClientEvent::OnConnect(self.id.clone(), host, user, tx))
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
        match result {
            Ok(message) => {
                hub.send(ClientEvent::OnMessage(self.id, message))
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(())
            }
            Err(forward_error) => {
                hub.send(ClientEvent::OnClose(self.id))
                    .await
                    .map_err(|send_error| io::Error::new(io::ErrorKind::Other, send_error))?;
                Err(forward_error)
            }
        }
    }
}

async fn forward_hub_to_client<T>(
    result: Option<ServerEvent>,
    write_half: &mut WriteHalf<T>,
) -> io::Result<()>
where
    T: AsyncRead + AsyncWrite,
{
    let event = result.ok_or(io::Error::new(io::ErrorKind::Other, "missing event"))?;
    match event {
        ServerEvent::OnMessage(message) => {
            message.write(write_half).await?;
        }
    }

    Ok(())
}
