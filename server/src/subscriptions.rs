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
        id: &Uuid,
        msg: SubscriptionRequest,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) {
        if msg.is_add {
            self.add_subscription(id, msg.topic.as_str(), client_manager, notification_manager)
                .await;
        } else {
            self.remove_subscription(
                id,
                msg.topic.as_str(),
                client_manager,
                notification_manager,
                false,
            )
            .await;
        }
    }

    async fn add_subscription(
        &mut self,
        subscriber_id: &Uuid,
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
        subscriber_id: &Uuid,
        topic: &str,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
        is_subscriber_closed: bool,
    ) {
        let Some(subscribers) = self.subscriptions.get_mut(topic) else {
            return;
        };

        let Some(count) = subscribers.get_mut(&subscriber_id) else {
            return;
        };

        if is_subscriber_closed {
            *count = 0;
        } else {
            *count -= 1;
        }

        if *count == 0 {
            subscribers.remove(&subscriber_id);
            println!("removed all subscriptions for {subscriber_id} on {topic}")
        } else {
            println!("removed one subscription for {subscriber_id} on {topic}")
        }

        if subscribers.len() == 0 {
            self.subscriptions.remove(topic);
        }

        notification_manager
            .notify_listeners(subscriber_id, topic, false, client_manager)
            .await;
    }

    pub async fn handle_close(
        &mut self,
        closed_client_id: &Uuid,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) {
        let closed_client_topic_subscriptions = self.find_client_topics(closed_client_id);
        for topic in closed_client_topic_subscriptions {
            self.remove_subscription(
                closed_client_id,
                &topic,
                client_manager,
                notification_manager,
                true,
            )
            .await
        }
    }

    fn find_client_topics(&self, client_id: &Uuid) -> Vec<String> {
        let mut topics: Vec<String> = Vec::new();
        for (topic, subscribers) in &self.subscriptions {
            if subscribers.contains_key(client_id) {
                topics.push(topic.clone());
            }
        }
        topics
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
