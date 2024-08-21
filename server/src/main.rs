use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;

use authentication::AuthenticationManager;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::{self, Sender};

use tokio_rustls::TlsAcceptor;

mod authentication;

mod authorization;
use authorization::{load_authorizations, AuthorizationSpec};

mod clients;

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

    // Command line options.
    let options = Options::load();

    let authorizations =
        load_authorizations(&options.authorizations_file, &options.authorizations)?;
    let authentication_manager = AuthenticationManager::new(&options.pwfile);

    // If using TLS create an acceptor.
    let tls_acceptor = match options.tls {
        true => Some(create_acceptor(options.certfile, options.keyfile)?),
        false => None,
    };

    // Create the listener.
    log::info!(
        "Listening on {}{}",
        options.endpoint.clone(),
        match options.tls {
            true => " using TLS",
            false => "",
        }
    );
    let addr = options
        .endpoint
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let listener = TcpListener::bind(&addr).await?;

    // Make the channel for the client-to-server communication.
    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);

    // Start the hub message processor. Note that is takes the receive end of
    // the mpsc channel.
    tokio::spawn(async move {
        Hub::run(authorizations, server_rx).await.unwrap();
    });

    handle_auth_reset(
        options.authorizations_file.clone(),
        options.authorizations.clone(),
        client_tx.clone(),
    )
    .await;

    loop {
        // Wait for a client to connect.
        let (stream, addr) = listener.accept().await?;

        // Start an interactor.
        spawn_interactor(
            stream,
            addr,
            tls_acceptor.clone(),
            client_tx.clone(),
            authentication_manager.clone(),
        )
        .await;
    }
}

async fn handle_auth_reset(
    authorizations_file: Option<PathBuf>,
    authorizations: Vec<AuthorizationSpec>,
    client_tx: Sender<ClientEvent>,
) {
    let mut hangup_stream = signal(SignalKind::hangup()).unwrap();
    tokio::spawn(async move {
        loop {
            // Wait for SIGHUP.
            hangup_stream.recv().await.unwrap();

            log::info!("Reloading authorizations");
            let authorizations =
                load_authorizations(&authorizations_file, &authorizations).unwrap();
            client_tx
                .send(ClientEvent::OnReset(authorizations))
                .await
                .unwrap();
        }
    });
}

async fn spawn_interactor(
    stream: TcpStream,
    addr: SocketAddr,
    tls_acceptor: Option<TlsAcceptor>,
    client_tx: Sender<ClientEvent>,
    authentication_manager: AuthenticationManager,
) {
    tokio::spawn(async move {
        let interactor = Interactor::new();

        let result = match tls_acceptor {
            Some(acceptor) => match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    println!("Connecting client {} over TLS", &interactor.id);
                    interactor
                        .run(tls_stream, addr, client_tx, authentication_manager)
                        .await
                }
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to create TLS acceptor{}", e),
                )),
            },
            None => {
                println!("Connecting client {}", &interactor.id);
                interactor
                    .run(stream, addr, client_tx, authentication_manager)
                    .await
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
