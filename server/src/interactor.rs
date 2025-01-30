use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use common::frame::{FrameReader, FrameWriter};
use tokio::io::{AsyncRead, AsyncWrite, BufReader, WriteHalf};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::RwLock;

use uuid::Uuid;

use common::messages::Message;

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
                result = FrameReader::read(&mut reader) => {
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
        result: Result<FrameReader, std::io::Error>,
        hub: &Sender<ClientEvent>,
    ) -> io::Result<()> {
        match result {
            Ok(mut frame_reader) => {
                let message = Message::deserialize(&mut frame_reader)?;
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
            let mut writer = FrameWriter::new();
            message.serialize(&mut writer)?;
            writer.write(write_half).await?;
        }
    }

    Ok(())
}
