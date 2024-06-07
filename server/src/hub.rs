use tokio::sync::mpsc::Receiver;

use uuid::Uuid;

use common::messages::Message;

use crate::{
    clients::ClientManager, events::ClientEvent, notifications::NotificationManager,
    publishing::PublisherManager, subscriptions::SubscriptionManager,
};

pub struct Hub {
    client_manager: ClientManager,
    subscription_manager: SubscriptionManager,
    notification_manager: NotificationManager,
    publisher_manager: PublisherManager,
}

impl Hub {
    pub fn new() -> Hub {
        Hub {
            client_manager: ClientManager::new(),
            subscription_manager: SubscriptionManager::new(),
            notification_manager: NotificationManager::new(),
            publisher_manager: PublisherManager::new(),
        }
    }

    pub async fn run(&mut self, mut server_rx: Receiver<ClientEvent>) {
        loop {
            let msg = server_rx.recv().await.unwrap();
            match msg {
                ClientEvent::OnConnect(id, host, user, server_tx) => self
                    .client_manager
                    .handle_connect(id, host, user, server_tx),
                ClientEvent::OnClose(id) => {
                    self.client_manager
                        .handle_close(
                            &id,
                            &mut self.subscription_manager,
                            &mut self.notification_manager,
                            &mut self.publisher_manager,
                        )
                        .await
                }
                ClientEvent::OnMessage(id, msg) => self.handle_message(id, msg).await,
            }
        }
    }

    async fn handle_message(&mut self, id: Uuid, msg: Message) {
        println!("Received message from {id}: \"{msg:?}\"");

        match msg {
            Message::AuthorizationRequest(_) => todo!(),
            Message::AuthorizationResponse(_) => todo!(),
            Message::ForwardedMulticastData(_) => todo!(),
            Message::ForwardedSubscriptionRequest(_) => todo!(),
            Message::ForwardedUnicastData(_) => todo!(),
            Message::MulticastData(msg) => {
                self.publisher_manager
                    .handle_multicast_data(
                        &id,
                        msg.topic,
                        msg.content_type,
                        msg.data_packets,
                        &self.subscription_manager,
                        &self.client_manager,
                    )
                    .await
            }
            Message::NotificationRequest(msg) => {
                self.notification_manager
                    .handle_notification_request(
                        &id,
                        msg,
                        &self.client_manager,
                        &self.subscription_manager,
                    )
                    .await
            }
            Message::SubscriptionRequest(msg) => {
                self.subscription_manager
                    .handle_subscription_request(
                        &id,
                        msg,
                        &self.client_manager,
                        &self.notification_manager,
                    )
                    .await
            }
            Message::UnicastData(msg) => {
                self.publisher_manager
                    .handle_unicast_data(
                        id,
                        msg.client_id,
                        msg.topic,
                        msg.content_type,
                        msg.data_packets,
                        &self.client_manager,
                    )
                    .await
            }
        }
    }
}
