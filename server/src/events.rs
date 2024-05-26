use std::net::SocketAddr;

use tokio::sync::mpsc::Sender;

pub enum ClientEvent {
    OnConnect(SocketAddr, Sender<ServerEvent>),
    OnMessage(SocketAddr, String)
}

pub enum ServerEvent {
    OnMessage(String),
}
