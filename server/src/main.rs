use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Sender};

use tokio_rustls::TlsAcceptor;

mod authentication;

mod authorization;

mod clients;

mod config;
use config::Config;

mod events;
use events::ClientEvent;

mod hub;
use hub::Hub;

mod interactor;
use interactor::Interactor;

mod options;
use options::Options;

mod notifications;

mod publishing;

mod subscriptions;

mod tls;
use tls::create_acceptor;

/*
 * The server starts by creating a `hub` task to process messages. It then
 * listens for client connections. When a client connects an interactor is
 * created.
 */
#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let options = Options::load();

    let config = match options.config {
        Some(path) => Config::load(&path).expect("Should load config"),
        None => Config::default(),
    };

    let addr = config
        .endpoint
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;

    let tls_acceptor = match config.tls.is_enabled {
        true => Some(create_acceptor(&config)?),
        false => None,
    };

    log::info!(
        "Listening on {}{}",
        config.endpoint.clone(),
        match config.tls.is_enabled {
            true => " using TLS",
            false => "",
        }
    );
    let listener = TcpListener::bind(&addr).await?;

    // Make the channel for the client-to-server communication.
    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);

    // Start the message processor.
    tokio::spawn(async move {
        Hub::run(config, server_rx).await.unwrap();
    });

    loop {
        // Wait for a client to connect.
        let (stream, addr) = listener.accept().await?;

        // Start an interactor.
        spawn_interactor(stream, addr, tls_acceptor.clone(), client_tx.clone()).await;
    }
}

async fn spawn_interactor(
    stream: TcpStream,
    addr: SocketAddr,
    tls_acceptor: Option<TlsAcceptor>,
    client_tx: Sender<ClientEvent>,
) {
    tokio::spawn(async move {
        let interactor = Interactor::new();

        let result = match tls_acceptor {
            Some(acceptor) => match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    println!("Connecting client {} over TLS", &interactor.id);
                    interactor.run(tls_stream, addr, client_tx).await
                }
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to create TLS acceptor{}", e),
                )),
            },
            None => {
                println!("Connecting client {}", &interactor.id);
                interactor.run(stream, addr, client_tx).await
            }
        };

        match result {
            Ok(()) => log::debug!("Client exited normally"),
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    log::debug!("Client closed connection")
                } else {
                    log::error!("Client exited with {e}")
                }
            }
        }
    });
}
