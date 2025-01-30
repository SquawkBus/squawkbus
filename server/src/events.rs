use tokio::sync::mpsc::Sender;

use common::messages::Message;

use crate::authorization::AuthorizationSpec;

pub enum ClientEvent {
    OnConnect(String, String, String, Sender<ServerEvent>),
    OnClose(String),
    OnMessage(String, Message),
    OnReset(Vec<AuthorizationSpec>),
}

pub enum ServerEvent {
    OnMessage(Message),
}
