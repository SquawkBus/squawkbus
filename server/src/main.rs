use std::io;

use config::Config;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use env_logger::Env;

mod events;
use events::ClientEvent;

mod hub;
use hub::Hub;

mod interactor;
use interactor::Interactor;

mod clients;
mod config;
mod entitlements;
mod notifications;
mod publishing;
mod subscriptions;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    // env_logger::init();

    let config = Config::load("etc/config-simple.yaml").expect("Should read config");

    log::info!("Listening on {}", config.endpoint.clone());
    let listener = TcpListener::bind(config.endpoint.clone()).await?;

    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);

    // Create a hub that listens to clients
    tokio::spawn(async move {
        Hub::run(config, server_rx).await.unwrap();
    });

    loop {
        let (socket, addr) = listener.accept().await?;

        let client_tx = client_tx.clone();
        let interactor = Interactor::new();

        tokio::spawn(async move {
            match interactor.run(socket, addr, client_tx).await {
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
}
