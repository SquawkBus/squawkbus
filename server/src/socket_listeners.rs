//! A real time message bus.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender;
use tokio_rustls::TlsAcceptor;

use common::{MessageSocket, MessageWebSocket};

use crate::{authentication::AuthenticationManager, events::ClientEvent, interactor::Interactor};

pub async fn spawn_interactor(
    is_web_socket: bool,
    stream: TcpStream,
    addr: SocketAddr,
    heartbeat_seconds: u64,
    tls_acceptor: Option<TlsAcceptor>,
    client_tx: Sender<ClientEvent>,
    authentication_manager: Arc<RwLock<AuthenticationManager>>,
) {
    tokio::spawn(async move {
        let result = start_interactor(
            is_web_socket,
            stream,
            addr,
            heartbeat_seconds,
            tls_acceptor,
            client_tx,
            authentication_manager,
        )
        .await;

        match result {
            Ok(()) => log::info!("Client exited normally."),
            Err(error) => {
                if error.kind() == io::ErrorKind::UnexpectedEof {
                    log::info!("Client closed connection.")
                } else {
                    log::info!("Client faulted with error: {error}")
                }
            }
        }
    });
}

async fn start_interactor(
    is_web_socket: bool,
    stream: TcpStream,
    addr: SocketAddr,
    heartbeat_seconds: u64,
    tls_acceptor: Option<TlsAcceptor>,
    client_tx: Sender<ClientEvent>,
    authentication_manager: Arc<RwLock<AuthenticationManager>>,
) -> io::Result<()> {
    let mut interactor = Interactor::new();

    match tls_acceptor {
        Some(acceptor) => {
            let stream = acceptor.accept(stream).await?;
            match is_web_socket {
                true => {
                    log::info!("Accepting web socket connection on adress {addr} over TLS.");
                    let stream = tokio_tungstenite::accept_async(stream).await.map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("failed to accept websocket: {}", e),
                        )
                    })?;
                    let mut stream = MessageWebSocket::new(stream);
                    interactor
                        .run(
                            &mut stream,
                            addr,
                            client_tx,
                            authentication_manager,
                            heartbeat_seconds,
                        )
                        .await
                }
                false => {
                    log::info!("Accepting socket connection on address {addr} over TLS.");
                    let mut stream = MessageSocket::new(stream);
                    interactor
                        .run(
                            &mut stream,
                            addr,
                            client_tx,
                            authentication_manager,
                            heartbeat_seconds,
                        )
                        .await
                }
            }
        }
        None => match is_web_socket {
            true => {
                log::info!("Accepting web socket connection on address {addr}.");
                let stream = tokio_tungstenite::accept_async(stream).await.map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("failed to accept websocket: {}", e),
                    )
                })?;
                let mut stream = MessageWebSocket::new(stream);
                interactor
                    .run(
                        &mut stream,
                        addr,
                        client_tx,
                        authentication_manager,
                        heartbeat_seconds,
                    )
                    .await
            }
            false => {
                log::info!("Accepting socket connection on address {addr}.");
                let mut stream = MessageSocket::new(stream);
                interactor
                    .run(
                        &mut stream,
                        addr,
                        client_tx,
                        authentication_manager,
                        heartbeat_seconds,
                    )
                    .await
            }
        },
    }
}
