use tokio::sync::mpsc::Sender;

use uuid::Uuid;

use common::messages::Message;

use crate::authorization::AuthorizationSpec;

pub enum ClientEvent {
    OnConnect(Uuid, String, String, Sender<ServerEvent>),
    OnClose(Uuid),
    OnMessage(Uuid, Message),
    OnReset(Vec<AuthorizationSpec>),
}

pub enum ServerEvent {
    OnMessage(Message),
}
