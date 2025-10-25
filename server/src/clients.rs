use std::collections::HashMap;
use std::io;

use tokio::sync::mpsc::Sender;

use crate::authorization::AuthorizationManager;
use crate::events::ServerEvent;
use crate::publishing::PublisherManager;
use crate::subscriptions::SubscriptionManager;

pub struct Client {
    pub tx: Sender<ServerEvent>,
    pub host: String,
    pub user: String,
}

pub struct ClientManager {
    clients: HashMap<String, Client>,
}

impl ClientManager {
    pub fn new() -> ClientManager {
        ClientManager {
            clients: HashMap::new(),
        }
    }

    pub fn handle_connect(
        &mut self,
        client_id: &str,
        host: String,
        user: String,
        tx: Sender<ServerEvent>,
    ) {
        log::debug!("client {client_id} connected for {user}@{host}");
        self.clients
            .insert(client_id.to_string(), Client { host, user, tx });
    }

    pub async fn handle_close(
        &mut self,
        client_id: &str,
        subscription_manager: &mut SubscriptionManager,
        publisher_manager: &mut PublisherManager,
        authorization_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        log::debug!("ClientManager::handle_close: closing {client_id}");

        subscription_manager
            .handle_close(client_id, self, publisher_manager, authorization_manager)
            .await?;

        publisher_manager
            .handle_close(client_id, self, subscription_manager)
            .await?;

        self.clients.remove(client_id);

        Ok(())
    }

    pub fn get(&self, client_id: &str) -> Option<&Client> {
        self.clients.get(client_id)
    }
}
