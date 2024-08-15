use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::net::SocketAddr;

use config::Config;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Sender};

use tokio_native_tls::native_tls::Identity;

mod events;
use events::ClientEvent;

mod hub;
use hub::Hub;

mod interactor;
use interactor::Interactor;
use tokio_native_tls::TlsAcceptor;

mod clients;
mod config;
mod entitlements;
mod notifications;
mod publishing;
mod subscriptions;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("usage: {} <config-file>", args[0])
    }

    let config = Config::load(&args[1]).expect("Should read config");

    let tls_acceptor = match config.tls.is_enabled {
        true => Some(create_acceptor(&config.tls.identity)),
        false => None,
    };

    log::info!("Listening on {}", config.endpoint.clone());
    let listener = TcpListener::bind(config.endpoint.clone()).await?;

    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);

    // Create a hub that listens to clients
    tokio::spawn(async move {
        Hub::run(config, server_rx).await.unwrap();
    });

    loop {
        let (stream, addr) = listener.accept().await?;
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

fn create_acceptor(path: &String) -> tokio_native_tls::TlsAcceptor {
    let mut file = File::open(path).expect("Should open certificate");
    let mut identity = vec![];
    file.read_to_end(&mut identity)
        .expect("Should read certificate");
    let identity = Identity::from_pkcs12(&identity, "trustno1").expect("Should create identity");

    tokio_native_tls::TlsAcceptor::from(
        native_tls::TlsAcceptor::builder(identity)
            .build()
            .expect("Should build native acceptor"),
    )
}
