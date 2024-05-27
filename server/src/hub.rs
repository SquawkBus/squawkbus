use std::collections::HashMap;
use std::sync::Arc;

use regex::Regex;
use tokio::sync::mpsc::{Receiver, Sender};

use uuid::Uuid;

use common::messages::{ForwardedSubscriptionRequest, Message, MulticastData, NotificationRequest, SubscriptionRequest};

use crate::events::{ClientEvent, ServerEvent};


pub struct Hub {
    clients: HashMap<Uuid, Client>,
    subscriptions: HashMap<String, HashMap<Uuid, u32>>,
    notifications: HashMap<String, (Regex, HashMap<Uuid, u32>)>
}

struct Client {
    pub tx: Sender<Arc<ServerEvent>>,
    pub host: String,
    pub user: String
}

impl Hub {
    pub fn new() -> Hub {
        Hub {
            clients: HashMap::new(),
            subscriptions: HashMap::new(),
            notifications: HashMap::new()
        }
    }

    pub async fn run(&mut self, mut server_rx: Receiver<ClientEvent>) {
        loop {
            let msg = server_rx.recv().await.unwrap();
            match msg {
                ClientEvent::OnConnect(id, host, user, server_tx) => self.handle_connect(id, host, user, server_tx),
                ClientEvent::OnMessage(id, msg) => self.handle_message(id, msg).await,
            }
        }    
    }

    fn handle_connect(&mut self, id: Uuid, host: String, user: String, tx: Sender<Arc<ServerEvent>>) {
        println!("client connected from {id}");
        self.clients.insert(id, Client { host, user, tx });
    }

    async fn handle_message(&mut self, id: Uuid, msg: Message) {
        println!("Received message from {id}: \"{msg:?}\"");

        match msg {
            Message::AuthorizationRequest(_) => todo!(),
            Message::AuthorizationResponse(_) => todo!(),
            Message::ForwardedMulticastData(_) => todo!(),
            Message::ForwardedSubscriptionRequest(msg) => todo!(),
            Message::ForwardedUnicastData(_) => todo!(),
            Message::MulticastData(msg) => self.handle_multicast_data(msg).await,
            Message::NotificationRequest(msg) => self.handle_notification_request(&id, msg).await,
            Message::SubscriptionRequest(msg) => self.handle_subscription_request(&id, msg).await,
            Message::UnicastData(_) => todo!(),
        }
    }

    async fn handle_notification_request(&mut self, id: &Uuid, msg: NotificationRequest) {
        if msg.is_add {
            self.add_notification(id, msg.pattern.as_str()).await;
        } else {
            self.remove_notification(id, msg.pattern.as_str()).await;
        }
    }

    async fn add_notification(&mut self, listener_id: &Uuid, pattern: &str) {
        let (regex, listeners) = self.notifications.entry(pattern.to_string()).or_insert((Regex::new(pattern).unwrap(), HashMap::new()));

        if let Some(count) = listeners.get_mut(&listener_id) {
            *count += 1;
        } else {
            listeners.insert(listener_id.clone(), 1);
        }

        for (topic, subscribers) in &self.subscriptions {
            if regex.is_match(topic.as_str()) {
                for subscriber_id in subscribers.keys() {
                    let client = self.clients.get(subscriber_id).unwrap();
                    let message = ForwardedSubscriptionRequest {
                        client_id: subscriber_id.clone(),
                        host: client.host.clone(),
                        user: client.user.clone(),
                        topic: topic.clone(),
                        is_add: true
                    };
                    let event = Arc::new(ServerEvent::OnMessage(Message::ForwardedSubscriptionRequest(message)));
                    client.tx.send(event).await.unwrap();
                }
            }
        }
    }

    async fn remove_notification(&mut self, listener_id: &Uuid, pattern: &str) {
        let Some((_, listeners)) = self.notifications.get_mut(pattern) else {
            return
        };

        let Some(count) = listeners.get_mut(listener_id) else {
            return
        };

        *count -= 1;

        if *count == 0 {
            listeners.remove(&listener_id);
        }

        if listeners.len() == 0 {
            self.notifications.remove(pattern);
        }
    }

    async fn handle_subscription_request(&mut self, id: &Uuid, msg: SubscriptionRequest) {
        if msg.is_add {
            self.add_subscription(id, msg.topic.as_str()).await;
        } else {
            self.remove_subscription(id, msg.topic.as_str()).await;
        }
    }

    async fn add_subscription(&mut self, subscriber_id: &Uuid, topic: &str) {
        let subscribers = self.subscriptions.entry(topic.to_string()).or_default();

        if let Some(count) = subscribers.get_mut(&subscriber_id) {
            *count += 1;
        } else {
            subscribers.insert(subscriber_id.clone(), 1);
            self.notify_listeners(subscriber_id, topic, true).await;
        }
    }

    async fn remove_subscription(&mut self, subscriber_id: &Uuid, topic: &str) {
        let Some(subscribers) = self.subscriptions.get_mut(topic) else {
            return
        };

        let Some(count) = subscribers.get_mut(subscriber_id) else {
            return
        };

        *count -= 1;

        if *count == 0 {
            subscribers.remove(&subscriber_id);
        }

        if subscribers.len() == 0 {
            self.subscriptions.remove(topic);
        }

        self.notify_listeners(subscriber_id, topic, false).await;
    }

    async fn handle_multicast_data(&mut self, msg: MulticastData) {

    }

    async fn notify_listeners(&self, subscriber_id: &Uuid, topic: &str, is_add: bool) {
        for (_pattern, (regex, listeners)) in &self.notifications {
            if regex.is_match(topic) {
                let subscriber = self.clients.get(subscriber_id).unwrap();

                let message = ForwardedSubscriptionRequest {
                    client_id: subscriber_id.clone(),
                    host: subscriber.host.clone(),
                    user: subscriber.user.clone(),
                    topic: topic.to_string(),
                    is_add
                };
                let event = Arc::new(ServerEvent::OnMessage(Message::ForwardedSubscriptionRequest(message)));
        
                for (listener_id, _) in listeners {
                    let listener = self.clients.get(listener_id).unwrap();
                    listener.tx.send(event.clone()).await.unwrap();
                }        
            }
        }
    }
}
