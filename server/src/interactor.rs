use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::mpsc::{self, Sender};
use tokio::sync::RwLock;

use uuid::Uuid;

use common::messages::Message;
use common::MessageStream;

use crate::authentication::AuthenticationManager;
use crate::events::{ClientEvent, ServerEvent};

#[derive(Debug)]
pub struct Interactor {
    pub id: String,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor {
            id: Uuid::new_v4().to_string(),
        }
    }

    pub async fn run(
        &self,
        stream: &mut impl MessageStream,
        addr: SocketAddr,
        hub: Sender<ClientEvent>,
        authentication_manager: Arc<RwLock<AuthenticationManager>>,
    ) -> io::Result<()> {
        let (tx, mut rx) = mpsc::channel::<ServerEvent>(32);

        let user = self.authenticate(stream, authentication_manager).await?;

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
                result = stream.read() => {
                    self.forward_client_to_hub(result, &hub).await
                }
                // forward hub to client
                result = rx.recv() => {
                    self.forward_hub_to_client(result, stream).await
                }
            }?
        }
    }

    async fn authenticate(
        &self,
        stream: &mut impl MessageStream,
        authentication_manager: Arc<RwLock<AuthenticationManager>>,
    ) -> io::Result<String> {
        // If successful, the authentication manager resolves the user for
        // authorization.
        // If unsuccessful an error will be returned and propagated up until
        // the connection is closed.
        let user = authentication_manager
            .read() // Acquire the lock.
            .await
            .authenticate(stream)
            .await?;

        // The id is returned to the client.
        let response = Message::AuthenticationResponse {
            client_id: self.id.clone(),
        };
        stream.write(&response).await?;

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

    async fn forward_hub_to_client(
        &self,
        event: Option<ServerEvent>,
        stream: &mut impl MessageStream,
    ) -> io::Result<()> {
        let event = event.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing event"))?;
        match event {
            ServerEvent::OnMessage(message) => {
                let client_id = &self.id;
                log::debug!("Sent message to {client_id}: \"{message:?}\"");
                stream.write(&message).await?;
            }
        }

        Ok(())
    }
}
