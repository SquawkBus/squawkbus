use std::collections::HashMap;

use regex::Regex;

use uuid::Uuid;

use common::messages::SubscriptionRequest;

use crate::{clients::ClientManager, notifications::NotificationManager};

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
