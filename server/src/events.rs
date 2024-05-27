use std::net::SocketAddr;
use std::sync::Arc;

use common::messages::Message;
use tokio::sync::mpsc::Sender;

pub enum ClientEvent {
    OnConnect(SocketAddr, Sender<Arc<ServerEvent>>),
    OnMessage(SocketAddr, Message)
}

pub enum ServerEvent {
    OnMessage(Message),
}
