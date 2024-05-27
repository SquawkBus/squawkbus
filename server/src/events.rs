use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use uuid::Uuid;

use common::messages::Message;

pub enum ClientEvent {
    OnConnect(Uuid, Sender<Arc<ServerEvent>>),
    OnMessage(Uuid, Message)
}

pub enum ServerEvent {
    OnMessage(Message),
}
