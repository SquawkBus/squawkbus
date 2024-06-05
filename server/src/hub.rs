use std::collections::HashMap;
use std::sync::Arc;

use regex::Regex;
use tokio::sync::mpsc::{Receiver, Sender};

use uuid::Uuid;

use common::messages::{
    DataPacket, ForwardedMulticastData, ForwardedSubscriptionRequest, ForwardedUnicastData,
    Message, NotificationRequest, SubscriptionRequest,
};

use crate::events::{ClientEvent, ServerEvent};

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

pub struct NotificationManager {
    notifications: HashMap<String, (Regex, HashMap<Uuid, u32>)>,
}

impl NotificationManager {
    pub fn new() -> NotificationManager {
        NotificationManager {
            notifications: HashMap::new(),
        }
    }

    pub async fn handle_notification_request(
        &mut self,
        id: &Uuid,
        msg: NotificationRequest,
        client_manager: &ClientManager,
        subscription_manager: &SubscriptionManager,
    ) {
        if msg.is_add {
            self.add_notification(
                id,
                msg.pattern.as_str(),
                client_manager,
                subscription_manager,
            )
            .await;
        } else {
            self.remove_notification(id, msg.pattern.as_str()).await;
        }
    }

    pub async fn add_notification(
        &mut self,
        listener_id: &Uuid,
        pattern: &str,
        client_manager: &ClientManager,
        subscription_manager: &SubscriptionManager,
    ) {
        let (regex, listeners) = self
            .notifications
            .entry(pattern.to_string())
            .or_insert((Regex::new(pattern).unwrap(), HashMap::new()));

        if let Some(count) = listeners.get_mut(&listener_id) {
            *count += 1;
        } else {
            listeners.insert(listener_id.clone(), 1);
        }

        for (topic, subscribers) in subscription_manager.find_subscriptions(regex) {
            if regex.is_match(topic.as_str()) {
                for subscriber_id in &subscribers {
                    let client = client_manager.get(subscriber_id).unwrap();
                    let message = ForwardedSubscriptionRequest {
                        client_id: subscriber_id.clone(),
                        host: client.host.clone(),
                        user: client.user.clone(),
                        topic: topic.clone(),
                        is_add: true,
                    };
                    let event = Arc::new(ServerEvent::OnMessage(
                        Message::ForwardedSubscriptionRequest(message),
                    ));
                    client.tx.send(event).await.unwrap();
                }
            }
        }
    }

    pub async fn remove_notification(&mut self, listener_id: &Uuid, pattern: &str) {
        let Some((_, listeners)) = self.notifications.get_mut(pattern) else {
            return;
        };

        let Some(count) = listeners.get_mut(listener_id) else {
            return;
        };

        *count -= 1;

        if *count == 0 {
            listeners.remove(&listener_id);
        }

        if listeners.len() == 0 {
            self.notifications.remove(pattern);
        }
    }

    async fn notify_listeners(
        &self,
        subscriber_id: Uuid,
        topic: &str,
        is_add: bool,
        client_manager: &ClientManager,
    ) {
        println!("notify_listeners: subscriber_id={subscriber_id}, topic={topic}, is_add={is_add}");

        for (_pattern, (regex, listeners)) in &self.notifications {
            if regex.is_match(topic) {
                let subscriber = client_manager.get(&subscriber_id).unwrap();

                let message = ForwardedSubscriptionRequest {
                    client_id: subscriber_id.clone(),
                    host: subscriber.host.clone(),
                    user: subscriber.user.clone(),
                    topic: topic.to_string(),
                    is_add,
                };
                let event = Arc::new(ServerEvent::OnMessage(
                    Message::ForwardedSubscriptionRequest(message),
                ));

                for (listener_id, _) in listeners {
                    let listener = client_manager.get(listener_id).unwrap();
                    listener.tx.send(event.clone()).await.unwrap();
                }
            }
        }
    }
}

pub struct SubscriptionManager {
    subscriptions: HashMap<String, HashMap<Uuid, u32>>,
}

impl SubscriptionManager {
    pub fn new() -> SubscriptionManager {
        SubscriptionManager {
            subscriptions: HashMap::new(),
        }
    }

    pub fn subscribers_for_topic(&self, topic: &str) -> Option<&HashMap<Uuid, u32>> {
        self.subscriptions.get(topic)
    }

    pub async fn handle_subscription_request(
        &mut self,
        id: Uuid,
        msg: SubscriptionRequest,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) {
        if msg.is_add {
            self.add_subscription(id, msg.topic.as_str(), client_manager, notification_manager)
                .await;
        } else {
            self.remove_subscription(id, msg.topic.as_str(), client_manager, notification_manager)
                .await;
        }
    }

    async fn add_subscription(
        &mut self,
        subscriber_id: Uuid,
        topic: &str,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) {
        let subscribers = self.subscriptions.entry(topic.to_string()).or_default();

        if let Some(count) = subscribers.get_mut(&subscriber_id) {
            println!("add_subscription: incrementing count for {topic}");
            *count += 1;
        } else {
            println!("add_subscription: creating new {topic}");
            subscribers.insert(subscriber_id.clone(), 1);
            notification_manager
                .notify_listeners(subscriber_id, topic, true, client_manager)
                .await;
        }
    }

    async fn remove_subscription(
        &mut self,
        subscriber_id: Uuid,
        topic: &str,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) {
        let Some(subscribers) = self.subscriptions.get_mut(topic) else {
            return;
        };

        let Some(count) = subscribers.get_mut(&subscriber_id) else {
            return;
        };

        *count -= 1;

        if *count == 0 {
            subscribers.remove(&subscriber_id);
        }

        if subscribers.len() == 0 {
            self.subscriptions.remove(topic);
        }

        notification_manager
            .notify_listeners(subscriber_id, topic, false, client_manager)
            .await;
    }

    pub fn find_subscriptions(&self, regex: &Regex) -> Vec<(String, Vec<Uuid>)> {
        let mut subscriptions: Vec<(String, Vec<Uuid>)> = Vec::new();
        for (topic, subscribers) in &self.subscriptions {
            if regex.is_match(topic.as_str()) {
                subscriptions.push((
                    topic.clone(),
                    subscribers.keys().map(|x| x.clone()).collect(),
                ));
            }
        }
        subscriptions
    }
}

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
                        id,
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
