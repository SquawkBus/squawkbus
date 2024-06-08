use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use uuid::Uuid;

use crate::events::ServerEvent;
use crate::notifications::NotificationManager;
use crate::publishing::PublisherManager;
use crate::subscriptions::SubscriptionManager;

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
        log::debug!("client {id} connected for {user}@{host}");
        self.clients.insert(id, Client { host, user, tx });
    }

    pub async fn handle_close(
        &mut self,
        id: &Uuid,
        subscription_manager: &mut SubscriptionManager,
        notification_manager: &mut NotificationManager,
        publisher_manager: &mut PublisherManager,
    ) -> io::Result<()> {
        log::debug!("ClientManager::handle_close: closing {id}");

        subscription_manager
            .handle_close(id, self, notification_manager)
            .await?;

        notification_manager.handle_close(id).await?;

        publisher_manager
            .handle_close(id, self, subscription_manager)
            .await?;

        self.clients.remove(&id);

        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Option<&Client> {
        self.clients.get(&id)
    }
}
