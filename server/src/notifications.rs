use std::collections::HashMap;
use std::sync::Arc;

use regex::Regex;

use uuid::Uuid;

use common::messages::{ForwardedSubscriptionRequest, Message, NotificationRequest};

use crate::{clients::ClientManager, events::ServerEvent, subscriptions::SubscriptionManager};

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
            self.remove_notification(id, msg.pattern.as_str(), false)
                .await;
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

    pub async fn remove_notification(
        &mut self,
        listener_id: &Uuid,
        pattern: &str,
        is_listener_closed: bool,
    ) {
        let Some((_, listeners)) = self.notifications.get_mut(pattern) else {
            return;
        };

        let Some(count) = listeners.get_mut(listener_id) else {
            return;
        };

        if is_listener_closed {
            *count = 0;
        } else {
            *count -= 1;
        }

        if *count == 0 {
            listeners.remove(&listener_id);
            println!("removed all notifications for {listener_id} on {pattern}")
        } else {
            println!("removed one notification for {listener_id} on {pattern}")
        }

        if listeners.len() == 0 {
            self.notifications.remove(pattern);
        }
    }

    pub async fn notify_listeners(
        &self,
        subscriber_id: &Uuid,
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

    pub async fn handle_close(&mut self, listener_id: &Uuid) {
        let patterns = self.find_listener_patterns(listener_id);
        for pattern in patterns {
            self.remove_notification(listener_id, &pattern, true).await
        }
    }

    fn find_listener_patterns(&self, listener_id: &Uuid) -> Vec<String> {
        let mut patterns: Vec<String> = Vec::new();
        for (pattern, (_regex, listeners)) in &self.notifications {
            if listeners.contains_key(listener_id) {
                patterns.push(pattern.clone());
            }
        }
        patterns
    }
}
