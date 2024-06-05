use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use uuid::Uuid;

use crate::events::ServerEvent;

pub struct Client {
    pub tx: Sender<Arc<ServerEvent>>,
    pub host: String,
    pub user: String,
}

pub struct ClientManager {
    clients: HashMap<Uuid, Client>,
}

impl ClientManager {
    pub fn new() -> ClientManager {
        ClientManager {
            clients: HashMap::new(),
        }
    }

    pub fn handle_connect(
        &mut self,
        id: Uuid,
        host: String,
        user: String,
        tx: Sender<Arc<ServerEvent>>,
    ) {
        println!("client connected from {id}");
        self.clients.insert(id, Client { host, user, tx });
    }

    pub fn get(&self, id: &Uuid) -> Option<&Client> {
        self.clients.get(&id)
    }
}
