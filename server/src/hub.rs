use std::io;
use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use common::messages::Message;

use crate::{
    authorization::{AuthorizationManager, AuthorizationSpec},
    clients::ClientManager,
    events::{ClientEvent, ServerEvent},
    publishing::PublisherManager,
    subscriptions::SubscriptionManager,
};

struct HubManager {
    client_manager: ClientManager,
    subscription_manager: SubscriptionManager,
    publisher_manager: PublisherManager,
    authorization_manager: AuthorizationManager,
}

impl HubManager {
    pub fn new(entitlement_manager: AuthorizationManager) -> Self {
        HubManager {
            client_manager: ClientManager::new(),
            subscription_manager: SubscriptionManager::new(),
            publisher_manager: PublisherManager::new(),
            authorization_manager: entitlement_manager,
        }
    }

    pub async fn handle_event(&mut self, event: ClientEvent) -> io::Result<()> {
        match event {
            ClientEvent::OnMessage(id, msg) => self.handle_message(&id, msg).await,
            ClientEvent::OnConnect(id, host, user, server_tx) => {
                Ok(self.handle_connect(&id, host, user, server_tx))
            }
            ClientEvent::OnClose(id) => self.handle_close(&id).await,
            ClientEvent::OnReset(specs) => Ok(self.handle_reset(specs)),
        }
    }

    fn handle_reset(&mut self, specs: Vec<AuthorizationSpec>) {
        log::debug!("Resetting authorizations");
        self.authorization_manager.reset(specs);
    }

    fn handle_connect(
        &mut self,
        client_id: &str,
        host: String,
        user: String,
        server_tx: Sender<ServerEvent>,
    ) {
        self.client_manager
            .handle_connect(client_id, host, user, server_tx)
    }

    async fn handle_close(&mut self, client_id: &str) -> io::Result<()> {
        self.client_manager
            .handle_close(
                client_id,
                &mut self.subscription_manager,
                &mut self.publisher_manager,
                &self.authorization_manager,
            )
            .await
    }

    async fn handle_message(&mut self, client_id: &str, msg: Message) -> io::Result<()> {
        log::debug!("Received message from {client_id}: \"{msg:?}\"");

        match msg {
            Message::MulticastData {
                topic,
                data_packets,
            } => {
                self.publisher_manager
                    .send_multicast_data(
                        client_id,
                        topic.as_str(),
                        data_packets,
                        &self.subscription_manager,
                        &self.client_manager,
                        &self.authorization_manager,
                    )
                    .await
            }
            Message::SubscriptionRequest { topic, is_add } => {
                self.subscription_manager
                    .handle_subscription_request(
                        &client_id,
                        &topic,
                        is_add,
                        &self.client_manager,
                        &mut self.publisher_manager,
                        &self.authorization_manager,
                    )
                    .await
            }
            Message::UnicastData {
                client_id: destination_id,
                topic,
                data_packets,
            } => {
                self.publisher_manager
                    .send_unicast_data(
                        client_id,
                        &destination_id,
                        topic.as_str(),
                        data_packets,
                        &self.client_manager,
                        &self.authorization_manager,
                    )
                    .await
            }
            _ => Err(io::Error::new(io::ErrorKind::Other, "unhandled message")),
        }
    }
}

pub struct Hub {
    state: Arc<Mutex<HubManager>>,
}

impl Hub {
    pub fn new(entitlement_manager: AuthorizationManager) -> Self {
        Hub {
            state: Arc::new(Mutex::new(HubManager::new(entitlement_manager))),
        }
    }
    pub async fn run(
        authorizations: Vec<AuthorizationSpec>,
        server_rx: Receiver<ClientEvent>,
    ) -> io::Result<()> {
        let mut hub_runner = Self::new(AuthorizationManager::new(authorizations));
        hub_runner.start(server_rx).await
    }

    async fn start(&mut self, mut server_rx: Receiver<ClientEvent>) -> io::Result<()> {
        loop {
            let msg = server_rx.recv().await.unwrap();
            let state = self.state.clone();
            let mut state = state.lock().await;
            state.handle_event(msg).await?
        }
    }
}
