use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, WriteHalf};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::RwLock;

use uuid::Uuid;

use common::messages::{AuthenticationResponse, Message};

use crate::authentication::AuthenticationManager;
use crate::events::{ClientEvent, ServerEvent};

#[derive(Debug)]
pub struct Interactor {
    pub id: String,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor {
            id: Uuid::new_v4().into(),
        }
    }

    pub async fn run<'a, T>(
        &self,
        socket: T,
        addr: SocketAddr,
        hub: Sender<ClientEvent>,
        authentication_manager: Arc<RwLock<AuthenticationManager>>,
    ) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let (tx, mut rx) = mpsc::channel::<ServerEvent>(32);

        let (read_half, mut write_half) = tokio::io::split(socket);

        let mut reader = BufReader::new(read_half);

        let user = self
            .authenticate(&mut reader, &mut write_half, authentication_manager)
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

    async fn authenticate<R, W>(
        &self,
        mut reader: &mut BufReader<R>,
        write_half: &mut WriteHalf<W>,
        authentication_manager: Arc<RwLock<AuthenticationManager>>,
    ) -> io::Result<String>
    where
        R: AsyncRead + Unpin,
        W: AsyncWriteExt + Unpin,
    {
        // If successful, the authentication manager resolves the user for
        // authorization.
        // If unsuccessful an error will be returned and propagated up until
        // the connection is closed.
        let user = authentication_manager
            .read() // Acquire the lock.
            .await
            .authenticate(&mut reader)
            .await?;

        // The id is returned to the client.
        let response = Message::AuthenticationResponse(AuthenticationResponse {
            client_id: self.id.clone(),
        });
        response.write(write_half).await?;

        Ok(user)
    }

    async fn forward_client_to_hub(
        &self,
        result: Result<Message, std::io::Error>,
        hub: &Sender<ClientEvent>,
    ) -> io::Result<()> {
        match result {
            Ok(message) => {
                hub.send(ClientEvent::OnMessage(self.id.clone(), message))
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(())
            }
            Err(forward_error) => {
                hub.send(ClientEvent::OnClose(self.id.clone()))
                    .await
                    .map_err(|send_error| io::Error::new(io::ErrorKind::Other, send_error))?;
                Err(forward_error)
            }
        }
    }
}

async fn forward_hub_to_client<T>(
    event: Option<ServerEvent>,
    write_half: &mut WriteHalf<T>,
) -> io::Result<()>
where
    T: AsyncRead + AsyncWrite,
{
    let event = event.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing event"))?;
    match event {
        ServerEvent::OnMessage(message) => {
            message.write(write_half).await?;
        }
    }

    Ok(())
}
