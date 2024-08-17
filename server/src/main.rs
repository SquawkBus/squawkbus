use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;

use pki_types::{CertificateDer, PrivateKeyDer};

use rustls_pemfile::{certs, private_key};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Sender};

use tokio_rustls::{rustls, TlsAcceptor};

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

    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);

    // Start the message processor.
    tokio::spawn(async move {
        Hub::run(config, server_rx).await.unwrap();
    });

    loop {
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
    let interactor = Interactor::new();

    tokio::spawn(async move {
        let result = match tls_acceptor {
            Some(acceptor) => match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    println!("Connecting TLS");
                    interactor.run(tls_stream, addr, client_tx).await
                }
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to create TLS acceptor{}", e),
                )),
            },
            None => {
                println!("Connecting PLAIN");
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

fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        ))?)
}

fn create_acceptor(config: &Config) -> io::Result<TlsAcceptor> {
    let certs = load_certs(&config.tls.certfile)?;
    let key = load_key(&config.tls.keyfile)?;
    // let flag_echo = options.echo_mode;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    Ok(acceptor)
}
