use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
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

    pub async fn run(&self, mut socket: TcpStream, addr: SocketAddr, hub: Sender<ClientEvent>) {
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
                    let msg = result.unwrap();
                    hub.send(ClientEvent::OnMessage(self.id, msg)).await.unwrap();
                }
                // forward hub to client
                result = rx.recv() => {
                    match result {
                        Some(event) => {
                            match event.as_ref() {
                                ServerEvent::OnMessage(message) => {
                                    message.write(&mut write_half).await.unwrap();
                                    // write_half.write_all(line.as_bytes()).await.unwrap();
                                }
                            }
                        },
                        None => todo!(),
                    }
                }
            }
        }
    }
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
