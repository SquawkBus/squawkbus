use std::{
    collections::{HashMap, HashSet},
    io,
};

use wildmatch::WildMatch;

use crate::{clients::ClientManager, notifications::NotificationManager};

struct Subscription {
    pattern: WildMatch,
    subscribers: HashMap<String, u32>,
}

impl Subscription {
    pub fn new(topic: &str) -> Self {
        Subscription {
            pattern: WildMatch::new(topic),
            subscribers: HashMap::new(),
        }
    }
}

pub struct SubscriptionManager {
    subscriptions: HashMap<String, Subscription>,
}

impl SubscriptionManager {
    pub fn new() -> SubscriptionManager {
        SubscriptionManager {
            subscriptions: HashMap::new(),
        }
    }

    pub fn subscribers_for_topic(&self, topic: &str) -> HashSet<String> {
        let mut subscribers: HashSet<String> = HashSet::new();

        for subscription in self.subscriptions.values() {
            if subscription.pattern.matches(topic) {
                for key in subscription.subscribers.keys() {
                    subscribers.insert(key.clone());
                }
            }
        }

        subscribers
    }

    pub async fn handle_subscription_request(
        &mut self,
        id: &str,
        topic: String,
        is_add: bool,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) -> io::Result<()> {
        if is_add {
            self.add_subscription(id, topic.as_str(), client_manager, notification_manager)
                .await
        } else {
            self.remove_subscription(
                id,
                topic.as_str(),
                client_manager,
                notification_manager,
                false,
            )
            .await
        }
    }

    async fn add_subscription(
        &mut self,
        subscriber_id: &str,
        topic: &str,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) -> io::Result<()> {
        // Add or get the subscription.
        if !self.subscriptions.contains_key(topic) {
            self.subscriptions
                .insert(topic.to_owned(), Subscription::new(topic));
        }
        let subscription = self.subscriptions.get_mut(topic).unwrap();

        // Keep a request count.
        if let Some(count) = subscription.subscribers.get_mut(subscriber_id) {
            log::debug!("add_subscription: incrementing count for {topic}");
            *count += 1;
        } else {
            log::debug!("add_subscription: creating new {topic}");
            subscription.subscribers.insert(subscriber_id.into(), 1);
            notification_manager
                .notify_listeners(subscriber_id, topic, true, client_manager)
                .await?;
        }

        Ok(())
    }

    async fn remove_subscription(
        &mut self,
        subscriber_id: &str,
        topic: &str,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
        is_subscriber_closed: bool,
    ) -> io::Result<()> {
        let Some(subscription) = self.subscriptions.get_mut(topic) else {
            return Ok(());
        };

        let Some(count) = subscription.subscribers.get_mut(subscriber_id) else {
            return Ok(());
        };

        if is_subscriber_closed {
            *count = 0;
        } else {
            *count -= 1;
        }

        if *count == 0 {
            subscription.subscribers.remove(subscriber_id);
            log::debug!("removed all subscriptions for {subscriber_id} on {topic}");
        } else {
            log::debug!("removed one subscription for {subscriber_id} on {topic}");
        }

        if subscription.subscribers.is_empty() {
            self.subscriptions.remove(topic);
        }

        notification_manager
            .notify_listeners(subscriber_id, topic, false, client_manager)
            .await
    }

    pub async fn handle_close(
        &mut self,
        closed_client_id: &str,
        client_manager: &ClientManager,
        notification_manager: &NotificationManager,
    ) -> io::Result<()> {
        let closed_client_topic_subscriptions = self.find_client_topics(closed_client_id);
        for topic in closed_client_topic_subscriptions {
            self.remove_subscription(
                closed_client_id,
                &topic,
                client_manager,
                notification_manager,
                true,
            )
            .await?;
        }

        Ok(())
    }

    fn find_client_topics(&self, client_id: &str) -> Vec<String> {
        let mut topics: Vec<String> = Vec::new();
        for (topic, subscription) in &self.subscriptions {
            if subscription.subscribers.contains_key(client_id) {
                topics.push(topic.clone());
            }
        }
        topics
    }

    pub fn find_subscriptions(&self, pattern: &WildMatch) -> Vec<(String, Vec<String>)> {
        let mut subscriptions: Vec<(String, Vec<String>)> = Vec::new();
        for (topic, subscription) in &self.subscriptions {
            if pattern.matches(topic.as_str()) {
                subscriptions.push((
                    topic.clone(),
                    subscription.subscribers.keys().map(|x| x.clone()).collect(),
                ));
            }
        }
        subscriptions
    }
}
