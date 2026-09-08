//! A real time message bus.

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::Arc;

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::RwLock;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;

mod socket_listeners;
use socket_listeners::spawn_interactor;

mod authentication;
use authentication::AuthenticationManager;

mod authorization;
use authorization::{AuthorizationSpec, load_authorizations};

mod clients;

mod events;
use events::ClientEvent;

mod hub;
use hub::Hub;

mod interactor;

mod config;
use config::Config;

mod notifications;

mod publishing;

mod http;
use http::start_http_server;

mod subscriptions;

mod tls;

use crate::tls::rustls_server_config;

/// The server starts by creating a `hub` task to process messages. It then
/// listens for client connections. When a client connects an interactor is
/// created.
#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let prometheus = setup_metrics_recorder()?;

    // Command line options.
    let options = Config::load()?;

    let authorizations =
        load_authorizations(&options.authorizations_file, &options.authorizations)?;
    let authentication_manager = Arc::new(RwLock::new(AuthenticationManager::new(
        &options.authentication,
    )?));

    // Make the channel for the client-to-server communication.
    let (client_tx, server_rx) = mpsc::channel::<ClientEvent>(32);

    let mut join_set = JoinSet::new();

    // Start the hub message processor. Note that is takes the receive end of
    // the mpsc channel.
    join_set.spawn(async move { Hub::run(authorizations, server_rx).await });

    handle_config_reset(
        options.authorizations_file.clone(),
        options.authorizations.clone(),
        authentication_manager.clone(),
        client_tx.clone(),
    )
    .await;

    let rustls_config = match options.tls {
        Some(tls_option) => Some(Arc::new(rustls_server_config(
            &tls_option.certfile,
            &tls_option.keyfile,
        )?)),
        None => None,
    };

    let tls_acceptor = match rustls_config.as_ref() {
        Some(socket_rustls_config) => Some(TlsAcceptor::from(socket_rustls_config.clone())),
        None => None,
    };

    let socket_addr = options
        .socket_endpoint
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let socket_tls_acceptor = tls_acceptor.clone();
    let socket_client_tx = client_tx.clone();
    let socket_authentication_manager = authentication_manager.clone();

    join_set.spawn(async move {
        start_listener(
            false,
            socket_addr,
            options.heartbeat_seconds,
            socket_tls_acceptor,
            socket_client_tx,
            socket_authentication_manager,
        )
        .await
    });

    let web_socket_addr = options
        .web_socket_endpoint
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let web_socket_tls_acceptor = tls_acceptor.clone();
    let web_socket_client_tx = client_tx.clone();
    let web_socket_authentication_manager = authentication_manager.clone();

    join_set.spawn(async move {
        start_listener(
            true,
            web_socket_addr,
            options.heartbeat_seconds,
            web_socket_tls_acceptor,
            web_socket_client_tx,
            web_socket_authentication_manager,
        )
        .await
    });

    let http_addr = options
        .http_endpoint
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let rest_rustls_config = rustls_config.as_ref().map(|x| x.clone());
    join_set
        .spawn(async move { start_http_server(http_addr, rest_rustls_config, prometheus).await });

    join_set.join_all().await;

    Ok(())
}

pub async fn start_listener(
    is_web_socket: bool,
    addr: SocketAddr,
    heartbeat_seconds: u64,
    tls_acceptor: Option<TlsAcceptor>,
    client_tx: Sender<ClientEvent>,
    authentication_manager: Arc<RwLock<AuthenticationManager>>,
) -> io::Result<()> {
    log::info!(
        "Listening on address {} for {}{}.",
        &addr,
        match is_web_socket {
            true => "web sockets",
            false => "sockets",
        },
        match tls_acceptor {
            Some(_) => " using TLS",
            None => "",
        }
    );

    let listener = TcpListener::bind(&addr).await?;

    loop {
        // Wait for a client to connect.
        let (stream, addr) = listener.accept().await?;

        // Start an interactor.
        spawn_interactor(
            is_web_socket,
            stream,
            addr,
            heartbeat_seconds,
            tls_acceptor.clone(),
            client_tx.clone(),
            authentication_manager.clone(),
        )
        .await;
    }
}

async fn handle_config_reset(
    authorizations_file: Option<PathBuf>,
    authorizations: Vec<AuthorizationSpec>,
    authentication_manager: Arc<RwLock<AuthenticationManager>>,
    client_tx: Sender<ClientEvent>,
) {
    let mut hangup_stream = signal(SignalKind::hangup()).unwrap();
    tokio::spawn(async move {
        loop {
            // Wait for SIGHUP.
            hangup_stream.recv().await.unwrap();

            authentication_manager.write().await.reset().await.unwrap();

            log::info!("Reloading authorizations.");
            let authorizations =
                load_authorizations(&authorizations_file, &authorizations).unwrap();
            client_tx
                .send(ClientEvent::OnReset(authorizations))
                .await
                .unwrap();
        }
    });
}

fn setup_metrics_recorder() -> io::Result<PrometheusHandle> {
    PrometheusBuilder::new()
        .install_recorder()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
}
