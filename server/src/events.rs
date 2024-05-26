use std::net::SocketAddr;

use common::messages::Message;
use tokio::sync::mpsc::Sender;

pub enum ClientEvent {
    OnConnect(SocketAddr, Sender<ServerEvent>),
    OnMessage(SocketAddr, Message)
}

pub enum ServerEvent {
    OnMessage(Message),
}
