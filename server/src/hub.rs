use std::sync::Arc;

use tokio::sync::mpsc::Receiver;

use uuid::Uuid;

use common::messages::{DataPacket, ForwardedMulticastData, ForwardedUnicastData, Message};

use crate::{
    clients::ClientManager,
    events::{ClientEvent, ServerEvent},
    notifications::NotificationManager,
    subscriptions::SubscriptionManager,
};

pub struct Hub {
    client_manager: ClientManager,
    subscription_manager: SubscriptionManager,
    notification_manager: NotificationManager,
}

impl Hub {
    pub fn new() -> Hub {
        Hub {
            client_manager: ClientManager::new(),
            subscription_manager: SubscriptionManager::new(),
            notification_manager: NotificationManager::new(),
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
                self.handle_multicast_data(&id, msg.topic, msg.content_type, msg.data_packets)
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
                self.handle_unicast_data(
                    id,
                    msg.client_id,
                    msg.topic,
                    msg.content_type,
                    msg.data_packets,
                )
                .await
            }
        }
    }

    async fn handle_unicast_data(
        &self,
        publisher_id: Uuid,
        client_id: Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) {
        let Some(publisher) = self.client_manager.get(&publisher_id) else {
            println!("handle_unicast_data: no publisher {publisher_id}");
            return;
        };

        let Some(client) = self.client_manager.get(&client_id) else {
            println!("handle_unicast_data: no client {client_id}");
            return;
        };

        let message = ForwardedUnicastData {
            client_id: publisher_id,
            host: publisher.host.clone(),
            user: publisher.user.clone(),
            topic,
            content_type,
            data_packets,
        };

        println!("handle_unicast_data: sending to client {client_id} message {message:?}");

        let event = Arc::new(ServerEvent::OnMessage(Message::ForwardedUnicastData(
            message,
        )));

        client.tx.send(event.clone()).await.unwrap();

        println!("handle_unicast_data: ...sent");
    }

    async fn handle_multicast_data(
        &self,
        publisher_id: &Uuid,
        topic: String,
        content_type: String,
        data_packets: Vec<DataPacket>,
    ) {
        let Some(subscribers) = self
            .subscription_manager
            .subscribers_for_topic(topic.as_str())
        else {
            println!("handle_multicast_data: no topic {topic}");
            return;
        };

        let Some(publisher) = self.client_manager.get(publisher_id) else {
            println!("handle_multicast_data: not publisher {publisher_id}");
            return;
        };

        let message = ForwardedMulticastData {
            host: publisher.host.clone(),
            user: publisher.user.clone(),
            topic,
            content_type,
            data_packets,
        };

        println!("handle_multicast_data: sending message {message:?} to clients ...");

        let event = Arc::new(ServerEvent::OnMessage(Message::ForwardedMulticastData(
            message,
        )));

        for subscriber_id in subscribers.keys() {
            if let Some(subscriber) = self.client_manager.get(subscriber_id) {
                println!("handle_multicast_data: ... {subscriber_id}");
                subscriber.tx.send(event.clone()).await.unwrap();
            }
        }

        println!("handle_multicast_data: ...sent");
    }
}
