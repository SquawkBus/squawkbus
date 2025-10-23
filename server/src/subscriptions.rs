use std::{
    collections::{HashMap, HashSet},
    io::{self, Cursor},
};

use common::{
    io::Serializable,
    messages::{DataPacket, Message},
};
use regex::Regex;

use crate::{
    authorization::AuthorizationManager, clients::ClientManager, publishing::PublisherManager,
};

const SUBSCRIPTION_TOPIC: &str = "__subscription__";

struct Subscription {
    regex: Regex,
    subscribers: HashMap<String, u32>,
}

impl Subscription {
    pub fn new(topic: &str) -> io::Result<Self> {
        let regex = Regex::new(topic).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Subscription {
            regex,
            subscribers: HashMap::new(),
        })
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
            if subscription.regex.is_match(topic) {
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
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        if is_add {
            self.add_subscription(
                id,
                topic.as_str(),
                client_manager,
                publisher_manager,
                entitlements_manager,
            )
            .await
        } else {
            self.remove_subscription(
                id,
                topic.as_str(),
                client_manager,
                publisher_manager,
                entitlements_manager,
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
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        // Add or get the subscription.
        if !self.subscriptions.contains_key(topic) {
            self.subscriptions
                .insert(topic.to_owned(), Subscription::new(topic)?);
        }
        let subscription = self.subscriptions.get_mut(topic).unwrap();

        // Keep a request count.
        if let Some(count) = subscription.subscribers.get_mut(subscriber_id) {
            log::debug!("add_subscription: incrementing count for {topic}");
            *count += 1;
        } else {
            log::debug!("add_subscription: creating new {topic}");
            subscription.subscribers.insert(subscriber_id.into(), 1);

            self.notify_subscription(
                subscriber_id,
                topic,
                true,
                client_manager,
                publisher_manager,
                entitlements_manager,
            )
            .await?;
        }

        Ok(())
    }

    async fn notify_subscription(
        &mut self,
        subscriber_id: &str,
        topic: &str,
        is_add: bool,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        if topic == SUBSCRIPTION_TOPIC {
            return Ok(());
        }

        let subscriber = client_manager.get(&subscriber_id).ok_or(io::Error::new(
            io::ErrorKind::Other,
            format!("unknown client {subscriber_id}"),
        ))?;

        let forwarded_subscription_request = Message::ForwardedSubscriptionRequest {
            host: subscriber.host.clone(),
            user: subscriber.user.clone(),
            client_id: subscriber_id.into(),
            topic: topic.to_string(),
            is_add,
        };

        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        forwarded_subscription_request
            .serialize(&mut cursor)
            .expect("should serialize");

        let data = cursor.into_inner();

        let data_packet = DataPacket {
            name: "forwarded_subscription_request".to_string(),
            entitlement: 0,
            content_type: "internal".to_string(),
            data,
        };

        publisher_manager
            .send_multicast_data(
                subscriber_id,
                SUBSCRIPTION_TOPIC,
                vec![data_packet],
                self,
                client_manager,
                entitlements_manager,
            )
            .await?;

        Ok(())
    }

    async fn remove_subscription(
        &mut self,
        subscriber_id: &str,
        topic: &str,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
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

        self.notify_subscription(
            subscriber_id,
            topic,
            false,
            client_manager,
            publisher_manager,
            entitlements_manager,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_close(
        &mut self,
        closed_client_id: &str,
        client_manager: &ClientManager,
        publisher_manager: &mut PublisherManager,
        entitlements_manager: &AuthorizationManager,
    ) -> io::Result<()> {
        let closed_client_topic_subscriptions = self.find_client_topics(closed_client_id);
        for topic in closed_client_topic_subscriptions {
            self.remove_subscription(
                closed_client_id,
                &topic,
                client_manager,
                publisher_manager,
                entitlements_manager,
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

    pub fn find_subscriptions(&self, regex: &Regex) -> Vec<(String, Vec<String>)> {
        let mut subscriptions: Vec<(String, Vec<String>)> = Vec::new();
        for (topic, subscription) in &self.subscriptions {
            if regex.is_match(topic.as_str()) {
                subscriptions.push((
                    topic.clone(),
                    subscription.subscribers.keys().map(|x| x.clone()).collect(),
                ));
            }
        }
        subscriptions
    }
}
